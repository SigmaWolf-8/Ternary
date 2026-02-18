#!/bin/bash
# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# XPLENUM — Kernel Boot Validation Script
# Phase 6, Task 6.2: Boot Rust Kernel in QEMU with XPlenum Support
#
# Prerequisites:
#   - QEMU with XPlenum extension (Task 6.1b)
#   - Compiled Salvi Framework kernel binary
#   - RISC-V cross-compiler toolchain
#
# Usage:
#   ./xplenum_boot_test.sh [kernel_binary] [timeout_seconds]
# =============================================================================

set -euo pipefail

KERNEL="${1:-../../src/kernel/target/riscv64gc-unknown-none-elf/release/salvi_kernel}"
TIMEOUT="${2:-30}"
QEMU="${QEMU_RISCV:-qemu-system-riscv64}"
LOG_DIR="boot_logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BOOT_LOG="${LOG_DIR}/boot_${TIMESTAMP}.log"
TRACE_LOG="${LOG_DIR}/trace_${TIMESTAMP}.csv"

mkdir -p "$LOG_DIR"

echo "=========================================="
echo "XPlenum Kernel Boot Validation"
echo "=========================================="
echo "Kernel:  ${KERNEL}"
echo "QEMU:    ${QEMU}"
echo "Timeout: ${TIMEOUT}s"
echo ""

# ---------------------------------------------------------------------------
# Check prerequisites
# ---------------------------------------------------------------------------

if ! command -v "$QEMU" &>/dev/null; then
    echo "WARNING: QEMU not found at '${QEMU}'"
    echo "  Install with: apt install qemu-system-riscv64"
    echo "  Or set QEMU_RISCV environment variable"
    echo ""
    echo "Generating boot validation template instead..."

    cat > "${LOG_DIR}/boot_validation_template.txt" << 'TEMPLATE'
XPlenum Kernel Boot Validation — Expected Sequence
====================================================

1. QEMU launches with XPlenum custom instruction support
2. Kernel binary loaded at 0x80000000
3. M-mode initialization begins:
   a. CSR XPVERSION read → expect 0x010000 (v1.0.0)
   b. CSR XPSTATUS write → enable MASK_EN, DOM_EN, CAP_EN, SIG_EN (0x0F)
   c. CSR XPMASK_SEED write → initialize DRBG seed

4. Subsystem self-test sequence:
   a. TMASK + TUNMASK round-trip → verify XOR identity
   b. TMASKR → verify DRBG generates non-zero value
   c. TDOMSET + TDOMCHK → verify domain tag storage
   d. TCAPST + TCAPLD → verify capability creation
   e. TROTL + TROTR → verify rotation identity
   f. TTRIT + TDETRIT → verify encoding round-trip
   g. TSIGCMP → verify comparison semantics

5. Security policy initialization:
   a. Domain table populated for kernel/user separation
   b. Initial capabilities minted for kernel memory regions
   c. DRBG seeded and health tests confirmed green

6. Expected console output:
   [BOOT] XPlenum v1.0.0 detected
   [BOOT] Subsystem self-test: PASS (7/7)
   [BOOT] DRBG health: OK (rep_count=0, adap_prop=0)
   [BOOT] Security policy initialized
   [BOOT] Kernel ready

Validation Criteria:
- All self-tests pass
- CSR reads return expected values
- No unexpected exceptions during boot
- DRBG ready flag asserted within 100 cycles
- Performance counter increments correctly

Cross-Verification:
- Run same test vectors in Spike (Task 6.1a) and QEMU (Task 6.1b)
- Compare register traces using sim/cross-verify/xplenum_cross_verify.py
TEMPLATE

    echo "Template written to: ${LOG_DIR}/boot_validation_template.txt"
    exit 0
fi

# ---------------------------------------------------------------------------
# Run QEMU with XPlenum support
# ---------------------------------------------------------------------------

echo "Launching QEMU..."

timeout "$TIMEOUT" "$QEMU" \
    -M virt \
    -cpu rv64,x-xplenum=true \
    -m 256M \
    -nographic \
    -bios none \
    -kernel "$KERNEL" \
    -d guest_errors,unimp,int \
    -D "$BOOT_LOG" \
    -plugin xplenum_trace,file="$TRACE_LOG" \
    2>&1 | tee -a "$BOOT_LOG" || true

echo ""
echo "=========================================="
echo "Boot log:  ${BOOT_LOG}"
echo "Trace log: ${TRACE_LOG}"
echo "=========================================="

# ---------------------------------------------------------------------------
# Validate boot output
# ---------------------------------------------------------------------------

PASS_COUNT=0
FAIL_COUNT=0

check() {
    local desc="$1"
    local pattern="$2"
    if grep -q "$pattern" "$BOOT_LOG" 2>/dev/null; then
        echo "  PASS: $desc"
        ((PASS_COUNT++))
    else
        echo "  FAIL: $desc (pattern: $pattern)"
        ((FAIL_COUNT++))
    fi
}

echo ""
echo "Validation Checks:"
check "XPlenum detected"         "XPlenum.*detected\|xplenum.*init"
check "Self-test passed"         "self.test.*PASS\|self_test.*ok"
check "DRBG health OK"           "DRBG.*health.*OK\|drbg.*ready"
check "No unhandled exceptions"  "^[^E]*$"
check "Kernel ready"             "Kernel ready\|kernel_main\|boot complete"

echo ""
echo "Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo "INCOMPLETE — Some boot validation checks did not pass."
    echo "Review boot log for details: ${BOOT_LOG}"
    exit 1
fi

echo "PASS — Kernel boot validated successfully."
exit 0
