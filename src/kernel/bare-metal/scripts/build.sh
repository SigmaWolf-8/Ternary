#!/usr/bin/env bash
# PlenumNET Ternary Kernel — Bare-Metal Build
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.

set -euo pipefail
cd "$(dirname "$0")/.."

MODE="${1:-debug}"

echo "-- PlenumNET Bare-Metal Build --"
echo "   Mode: ${MODE}"

if ! rustup show active-toolchain 2>/dev/null | grep -q nightly; then
    rustup toolchain install nightly --component rust-src
    rustup override set nightly
fi

if ! rustup component list --installed 2>/dev/null | grep -q rust-src; then
    rustup component add rust-src --toolchain nightly
fi

if [ "$MODE" = "release" ]; then
    cargo build --release
    BINARY="target/x86_64-unknown-none/release/ternary-kernel"
else
    cargo build
    BINARY="target/x86_64-unknown-none/debug/ternary-kernel"
fi

if [ -f "$BINARY" ]; then
    SIZE=$(stat -c%s "$BINARY" 2>/dev/null || stat -f%z "$BINARY" 2>/dev/null)
    echo ""
    echo "-- Build Complete --"
    echo "   Binary: ${BINARY}"
    echo "   Size:   ${SIZE} bytes"
    echo "   Boot:   ./scripts/qemu-run.sh ${BINARY}"
else
    echo "[ERROR] Build failed"
    exit 1
fi
