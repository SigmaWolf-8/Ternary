#!/usr/bin/env bash
# PlenumNET Ternary Kernel — QEMU Bare-Metal Boot Runner
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.

set -euo pipefail

KERNEL="${1:?Usage: qemu-run.sh <kernel-binary>}"
TIMEOUT="${QEMU_TIMEOUT:-30}"

if ! command -v qemu-system-x86_64 &>/dev/null; then
    echo "[ERROR] qemu-system-x86_64 not found."
    echo "  Install: sudo apt install qemu-system-x86 (Debian/Ubuntu)"
    echo "           brew install qemu (macOS)"
    exit 2
fi

echo "================================================================"
echo "  PlenumNET — QEMU Bare-Metal Validation"
echo "  Kernel:  ${KERNEL##*/}"
echo "  Target:  x86_64-unknown-none"
echo "  Timeout: ${TIMEOUT}s"
echo "================================================================"
echo ""

QEMU_EXIT_CODE=0
timeout "${TIMEOUT}" qemu-system-x86_64 \
    -kernel "${KERNEL}" \
    -serial stdio \
    -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -m 64M \
    || QEMU_EXIT_CODE=$?

echo ""

case $QEMU_EXIT_CODE in
    33)
        echo "================================================================"
        echo "  BARE-METAL VALIDATION: PASSED"
        echo "  The PlenumNET kernel boots on raw hardware."
        echo "================================================================"
        exit 0
        ;;
    35)
        echo "================================================================"
        echo "  BARE-METAL VALIDATION: FAILED"
        echo "================================================================"
        exit 1
        ;;
    124)
        echo "[ERROR] QEMU timed out after ${TIMEOUT}s"
        exit 1
        ;;
    *)
        echo "[ERROR] QEMU exited with code: ${QEMU_EXIT_CODE}"
        exit 1
        ;;
esac
