#!/usr/bin/env bash
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# PlenumNET Kernel — Build & Boot Script
#
# Usage:
#   ./boot.sh <arch> [--release] [--run] [--ci]
#
# Architectures:
#   x86_64   — Intel/AMD 64-bit (QEMU q35, multiboot2)
#   aarch64  — ARM 64-bit       (QEMU virt, PL011 UART)
#   riscv64  — RISC-V 64-bit    (QEMU virt, OpenSBI + NS16550)
#
# Flags:
#   --release   Build with release profile (LTO, opt-level 3)
#   --run       Launch QEMU after build
#   --ci        CI smoke-test mode: run QEMU with timeout, grep for sentinel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="${SCRIPT_DIR}/src/kernel"
SENTINEL="PLENUMNET KERNEL BOOT OK"

usage() {
    echo "Usage: $0 <x86_64|aarch64|riscv64> [--release] [--run] [--ci]"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

ARCH="$1"
shift

PROFILE="dev"
PROFILE_DIR="debug"
RUN=false
CI=false

while [ $# -gt 0 ]; do
    case "$1" in
        --release)
            PROFILE="release"
            PROFILE_DIR="release"
            ;;
        --run)
            RUN=true
            ;;
        --ci)
            CI=true
            RUN=true
            ;;
        *)
            echo "Unknown flag: $1"
            usage
            ;;
    esac
    shift
done

case "${ARCH}" in
    x86_64)
        TARGET="x86_64-unknown-none"
        RUSTFLAGS_EXTRA="-C code-model=small"
        ;;
    aarch64)
        TARGET="aarch64-unknown-none"
        RUSTFLAGS_EXTRA=""
        ;;
    riscv64)
        TARGET="riscv64gc-unknown-none-elf"
        RUSTFLAGS_EXTRA=""
        ;;
    *)
        echo "Error: unsupported architecture '${ARCH}'"
        echo "Supported: x86_64, aarch64, riscv64"
        exit 1
        ;;
esac

echo "================================================================"
echo "  PlenumNET Kernel Build"
echo "  Architecture : ${ARCH}"
echo "  Target       : ${TARGET}"
echo "  Profile      : ${PROFILE}"
echo "================================================================"
echo

BUILD_FLAGS=(
    --bin plenumnet-kernel
    --target "${TARGET}"
    --no-default-features
    --features no_std
)

if [ "${PROFILE}" = "release" ]; then
    BUILD_FLAGS+=(--release)
fi

export RUSTFLAGS="${RUSTFLAGS_EXTRA}"

echo "[build] cargo build ${BUILD_FLAGS[*]}"
(cd "${KERNEL_DIR}" && cargo build "${BUILD_FLAGS[@]}")

KERNEL_BIN="${KERNEL_DIR}/target/${TARGET}/${PROFILE_DIR}/plenumnet-kernel"

if [ ! -f "${KERNEL_BIN}" ]; then
    echo "Error: kernel binary not found at ${KERNEL_BIN}"
    exit 1
fi

echo
echo "[build] Kernel binary: ${KERNEL_BIN}"
echo "[build] Size: $(wc -c < "${KERNEL_BIN}") bytes"
echo

if [ "${RUN}" = false ]; then
    echo "Build complete. Use --run to boot in QEMU."
    exit 0
fi

QEMU_LOG=$(mktemp /tmp/plenumnet-qemu-XXXXXX.log)
trap "rm -f ${QEMU_LOG}" EXIT

QEMU_TIMEOUT=15

case "${ARCH}" in
    x86_64)
        QEMU_CMD=(
            qemu-system-x86_64
            -machine q35
            -cpu qemu64
            -m 128M
            -nographic
            -serial stdio
            -no-reboot
            -device isa-debugexit,iobase=0xf4,iosize=0x04
            -kernel "${KERNEL_BIN}"
        )
        ;;
    aarch64)
        QEMU_CMD=(
            qemu-system-aarch64
            -machine virt
            -cpu cortex-a57
            -m 128M
            -nographic
            -serial stdio
            -no-reboot
            -kernel "${KERNEL_BIN}"
        )
        ;;
    riscv64)
        QEMU_CMD=(
            qemu-system-riscv64
            -machine virt
            -cpu rv64
            -m 128M
            -nographic
            -serial stdio
            -no-reboot
            -bios default
            -kernel "${KERNEL_BIN}"
        )
        ;;
esac

echo "[qemu] ${QEMU_CMD[*]}"
echo

if [ "${CI}" = true ]; then
    set +e
    timeout "${QEMU_TIMEOUT}" "${QEMU_CMD[@]}" 2>&1 | tee "${QEMU_LOG}"
    QEMU_EXIT=$?
    set -e

    echo
    echo "[ci] QEMU exited with code ${QEMU_EXIT}"

    if grep -q "${SENTINEL}" "${QEMU_LOG}"; then
        echo "[ci] Sentinel found: ${SENTINEL}"
        echo "[ci] SMOKE TEST PASSED"
        exit 0
    else
        echo "[ci] Sentinel NOT found in QEMU output"
        echo "[ci] Expected: ${SENTINEL}"
        echo "[ci] SMOKE TEST FAILED"
        exit 1
    fi
else
    exec "${QEMU_CMD[@]}"
fi
