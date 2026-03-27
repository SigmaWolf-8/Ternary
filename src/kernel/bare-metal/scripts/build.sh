#!/usr/bin/env bash
# PlenumNET Ternary Kernel — Bare-Metal Build
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.
#
# Usage:
#   bash scripts/build.sh          # debug build
#   bash scripts/build.sh release  # release build

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "${SCRIPT_DIR}/.."

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

WRAPPER="${SCRIPT_DIR}/rustc-wrapper.sh"
export RUSTC="$WRAPPER"

if [ "${CLEAN:-}" = "1" ] || [ "${2:-}" = "clean" ]; then
    echo "[CLEAN] Removing previous build artifacts..."
    cargo clean 2>/dev/null || true
fi

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
find_objcopy() {
    # 1. llvm-objcopy from Rust toolchain (works on all hosts)
    local rustup_bin
    rustup_bin="$(find "$HOME/.rustup" -name 'llvm-objcopy' -type f 2>/dev/null | head -1)"
    if [ -n "$rustup_bin" ] && [ -x "$rustup_bin" ]; then echo "$rustup_bin"; return; fi
    # 2. Cross-binutils (essential on ARM64 hosts)
    if command -v x86_64-linux-gnu-objcopy &>/dev/null; then echo "x86_64-linux-gnu-objcopy"; return; fi
    # 3. System llvm-objcopy
    if command -v llvm-objcopy &>/dev/null; then echo "llvm-objcopy"; return; fi
    # 4. cargo-binutils wrapper
    if command -v rust-objcopy &>/dev/null; then echo "rust-objcopy"; return; fi
}
OBJCOPY="$(find_objcopy)"

if [ -z "$OBJCOPY" ]; then
    echo "[WARN] No suitable objcopy found."
    echo "  Fix: sudo apt install binutils-x86-64-linux-gnu"
    echo "  QEMU requires ELF32 for multiboot. Cannot convert."
    MB_BINARY="$BINARY"
else
    echo "[POST] Converting ELF64 → ELF32 (using $OBJCOPY)..."
    if $OBJCOPY --output-target=elf32-i386 "$BINARY" "$MB_BINARY" 2>/dev/null || \
       $OBJCOPY -O elf32-i386 "$BINARY" "$MB_BINARY" 2>/dev/null; then
        echo "[POST] Created: ${MB_BINARY}"
    else
        echo "[WARN] objcopy conversion failed. Using ELF64."
        MB_BINARY="$BINARY"
    fi
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
