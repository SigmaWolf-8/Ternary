#!/bin/bash
set -euo pipefail

echo "=========================================="
echo " PlenumNET Test Suite"
echo " Capomastro Holdings Ltd."
echo "=========================================="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

PASS=0
FAIL=0
SKIP=0

run_test_suite() {
    local name="$1"
    local dir="$2"
    local extra_args="${3:-}"

    echo "[TEST] $name..."
    if [ -f "$ROOT_DIR/$dir/Cargo.toml" ]; then
        if (cd "$ROOT_DIR/$dir" && cargo test $extra_args 2>&1); then
            echo "[PASS] $name"
            PASS=$((PASS + 1))
        else
            echo "[FAIL] $name"
            FAIL=$((FAIL + 1))
        fi
    else
        echo "[SKIP] $name (not found)"
        SKIP=$((SKIP + 1))
    fi
    echo ""
}

echo "--- Kernel Core Tests ---"
run_test_suite "Kernel (default features)" "src/kernel"
run_test_suite "Kernel (all features)" "src/kernel" "--all-features"
run_test_suite "Kernel (no_std)" "src/kernel" "--features no_std"
run_test_suite "Kernel (FINRA 613)" "src/kernel" "--features finra-613"

echo "--- Language Tool Tests ---"
run_test_suite "TSL Compiler" "src/tsl"
run_test_suite "THDL Synthesizer" "src/thdl"

echo ""
echo "=========================================="
echo " Test Results"
echo "=========================================="
echo " Passed:  $PASS"
echo " Failed:  $FAIL"
echo " Skipped: $SKIP"
echo "=========================================="

if [ $FAIL -gt 0 ]; then
    exit 1
fi
