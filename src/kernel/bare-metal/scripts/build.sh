#!/usr/bin/env bash
# PlenumNET Ternary Kernel — Bare-Metal Build
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.
#
# Usage:
#   bash scripts/build.sh          # debug build
#   bash scripts/build.sh release  # release build

set -euo pipefail
cd "$(dirname "$0")/.."

MODE="${1:-debug}"
TARGET="x86_64-unknown-none"

echo "================================================================"
echo "  PlenumNET — Bare-Metal Build"
echo "  Mode:   ${MODE}"
echo "  Target: ${TARGET}"
echo "  Host:   $(uname -m)"
echo "================================================================"

if ! command -v rustup &>/dev/null; then
    echo "[ERROR] rustup not found. Install Rust: https://rustup.rs"
    exit 1
fi

if ! rustup component list --installed 2>/dev/null | grep -q rust-src; then
    echo "[SETUP] Installing rust-src..."
    rustup component add rust-src
fi

WRAPPER="$(cd "$(dirname "$0")" && pwd)/rustc-wrapper.sh"
export RUSTC="$WRAPPER"

echo ""

BUILD_CMD="cargo build -Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem"

if [ "$MODE" = "release" ]; then
    $BUILD_CMD --release
    BINARY="target/${TARGET}/release/ternary-kernel"
else
    $BUILD_CMD
    BINARY="target/${TARGET}/debug/ternary-kernel"
fi

if [ -f "$BINARY" ]; then
    SIZE=$(stat -c%s "$BINARY" 2>/dev/null || stat -f%z "$BINARY" 2>/dev/null)
    echo ""
    echo "================================================================"
    echo "  BUILD COMPLETE"
    echo "  Binary: ${BINARY}"
    echo "  Size:   ${SIZE} bytes"
    echo ""
    echo "  Run QEMU test:"
    echo "    bash scripts/qemu-run.sh ${BINARY}"
    echo "================================================================"
else
    echo ""
    echo "[ERROR] Build failed — binary not found at ${BINARY}"
    exit 1
fi
