#!/usr/bin/env bash
# PlenumNET Ternary Kernel — Bare-Metal Build
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.
#
# Builds the bare-metal kernel binary for x86_64 QEMU validation.
# Uses the built-in x86_64-unknown-none target with -Zbuild-std passed
# on the command line to avoid cargo/rustc flag-injection issues across
# different nightly versions.
#
# Usage:
#   bash scripts/build.sh          # debug build
#   bash scripts/build.sh release  # release build

set -euo pipefail
cd "$(dirname "$0")/.."

MODE="${1:-debug}"
TARGET="x86_64-unknown-none"
BUILD_STD="-Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem"

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

if ! rustup show active-toolchain 2>/dev/null | grep -q nightly; then
    echo "[SETUP] Installing nightly toolchain..."
    rustup toolchain install nightly --component rust-src
    rustup override set nightly
fi

if ! rustup component list --installed 2>/dev/null | grep -q rust-src; then
    echo "[SETUP] Installing rust-src component..."
    rustup component add rust-src --toolchain nightly
fi

echo ""

if [ "$MODE" = "release" ]; then
    cargo $BUILD_STD build --release
    BINARY="target/${TARGET}/release/ternary-kernel"
else
    cargo $BUILD_STD build
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
