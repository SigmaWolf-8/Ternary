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

if [ ! -f "$BINARY" ]; then
    echo ""
    echo "[ERROR] Build failed — binary not found at ${BINARY}"
    exit 1
fi

MB_BINARY="${BINARY}.mb"

OBJCOPY=""
TOOLCHAIN_OBJCOPY="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | grep host | awk '{print $2}')/bin/llvm-objcopy"
if [ -x "$TOOLCHAIN_OBJCOPY" ]; then
    OBJCOPY="$TOOLCHAIN_OBJCOPY"
elif command -v llvm-objcopy &>/dev/null; then
    OBJCOPY="llvm-objcopy"
elif command -v rust-objcopy &>/dev/null; then
    OBJCOPY="rust-objcopy"
fi

if [ -z "$OBJCOPY" ]; then
    echo "[WARN] No objcopy found. Install: rustup component add llvm-tools"
    echo "       QEMU requires ELF32 for multiboot. Using ELF64 (may not boot)."
    MB_BINARY="$BINARY"
else
    echo "[POST] Converting ELF64 → ELF32 for QEMU multiboot..."
    $OBJCOPY -O elf32-i386 "$BINARY" "$MB_BINARY"
    echo "[POST] Created: ${MB_BINARY}"
fi

SIZE=$(stat -c%s "$MB_BINARY" 2>/dev/null || stat -f%z "$MB_BINARY" 2>/dev/null)
echo ""
echo "================================================================"
echo "  BUILD COMPLETE"
echo "  Binary: ${MB_BINARY}"
echo "  Size:   ${SIZE} bytes"
echo ""
echo "  Run QEMU test:"
echo "    bash scripts/qemu-run.sh ${MB_BINARY}"
echo "================================================================"
