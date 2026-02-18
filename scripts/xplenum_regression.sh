#!/bin/bash
# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# XPlenum Regression Test Framework
# Phase 3: Task 3.3 — Automated regression testing
#
# Runs: standalone testbench, integration testbench, formal verification
# Reports: pass/fail counts, timing, VCD generation
#
# Usage: ./scripts/xplenum_regression.sh [--standalone|--integration|--formal|--all]
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/results/regression_$(date +%Y%m%d_%H%M%S)"
LOG_FILE="$RESULTS_DIR/regression.log"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_SKIP=0

# -------------------------------------------------------------------------
# Utility functions
# -------------------------------------------------------------------------
log() { echo -e "$1" | tee -a "$LOG_FILE"; }

check_tool() {
    if command -v "$1" &>/dev/null; then
        log "  [OK] $1 found: $(command -v "$1")"
        return 0
    else
        log "  ${YELLOW}[SKIP]${NC} $1 not found"
        return 1
    fi
}

run_test() {
    local name="$1"
    local cmd="$2"
    local expected_pass="${3:-0}"

    log "\n${CYAN}--- Running: $name ---${NC}"

    local start_time=$(date +%s%N)
    local output
    local rc=0

    output=$(eval "$cmd" 2>&1) || rc=$?

    local end_time=$(date +%s%N)
    local elapsed_ms=$(( (end_time - start_time) / 1000000 ))

    echo "$output" >> "$RESULTS_DIR/${name// /_}.log"

    local pass_count=$(echo "$output" | grep -c '\[PASS\]' || true)
    local fail_count=$(echo "$output" | grep -c '\[FAIL\]' || true)

    if [[ $rc -eq 0 && $fail_count -eq 0 ]]; then
        log "${GREEN}[PASS]${NC} $name — ${pass_count} passed, ${fail_count} failed (${elapsed_ms}ms)"
        TOTAL_PASS=$((TOTAL_PASS + pass_count))
    else
        log "${RED}[FAIL]${NC} $name — ${pass_count} passed, ${fail_count} failed (${elapsed_ms}ms)"
        TOTAL_PASS=$((TOTAL_PASS + pass_count))
        TOTAL_FAIL=$((TOTAL_FAIL + fail_count))
    fi
}

# -------------------------------------------------------------------------
# Test suites
# -------------------------------------------------------------------------
run_standalone() {
    log "\n${CYAN}=== STANDALONE TESTBENCH ===${NC}"

    if ! check_tool iverilog; then
        log "  Skipping standalone tests (iverilog required)"
        TOTAL_SKIP=$((TOTAL_SKIP + 1))
        return
    fi

    local work_dir="$RESULTS_DIR/standalone"
    mkdir -p "$work_dir"

    iverilog -g2012 -I "$PROJECT_ROOT/rtl" \
        -o "$work_dir/xplenum_standalone_tb" \
        "$PROJECT_ROOT/rtl/xplenum_pkg.vh" \
        "$PROJECT_ROOT/rtl/xplenum_mask_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_domain_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_cap_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_trit_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_sig_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_top.v" \
        "$PROJECT_ROOT/tb/xplenum_tb.v" 2>&1 || {
        log "${RED}[FAIL]${NC} Standalone compile failed"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        return
    }

    run_test "Standalone TB" "cd $work_dir && vvp xplenum_standalone_tb"
}

run_integration() {
    log "\n${CYAN}=== INTEGRATION TESTBENCH ===${NC}"

    if ! check_tool iverilog; then
        log "  Skipping integration tests (iverilog required)"
        TOTAL_SKIP=$((TOTAL_SKIP + 1))
        return
    fi

    local work_dir="$RESULTS_DIR/integration"
    mkdir -p "$work_dir"

    iverilog -g2012 -I "$PROJECT_ROOT/rtl" \
        -o "$work_dir/xplenum_integration_tb" \
        "$PROJECT_ROOT/rtl/xplenum_pkg.vh" \
        "$PROJECT_ROOT/rtl/xplenum_mask_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_domain_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_cap_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_trit_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_sig_unit.v" \
        "$PROJECT_ROOT/rtl/xplenum_top.v" \
        "$PROJECT_ROOT/rtl/integration/xplenum_cva6_wrapper.v" \
        "$PROJECT_ROOT/rtl/integration/xplenum_stall_controller.v" \
        "$PROJECT_ROOT/rtl/integration/xplenum_cva6_top.v" \
        "$PROJECT_ROOT/tb/xplenum_cva6_integration_tb.v" 2>&1 || {
        log "${RED}[FAIL]${NC} Integration compile failed"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        return
    }

    run_test "Integration TB" "cd $work_dir && vvp xplenum_integration_tb"
}

run_formal() {
    log "\n${CYAN}=== FORMAL VERIFICATION ===${NC}"

    if ! check_tool sby; then
        log "  Skipping formal verification (sby/SymbiYosys required)"
        TOTAL_SKIP=$((TOTAL_SKIP + 1))
        return
    fi

    local work_dir="$RESULTS_DIR/formal"
    mkdir -p "$work_dir"

    # Standalone formal properties
    if [[ -f "$PROJECT_ROOT/rtl/formal/xplenum_formal.sby" ]]; then
        run_test "Formal Standalone BMC" \
            "cd $PROJECT_ROOT && sby -f rtl/formal/xplenum_formal.sby bmc 2>&1"
    fi

    # Integration formal properties
    if [[ -f "$PROJECT_ROOT/rtl/formal/xplenum_integration_formal.sby" ]]; then
        run_test "Formal Integration BMC" \
            "cd $PROJECT_ROOT && sby -f rtl/formal/xplenum_integration_formal.sby bmc 2>&1"
    fi
}

# -------------------------------------------------------------------------
# Main
# -------------------------------------------------------------------------
main() {
    mkdir -p "$RESULTS_DIR"

    log "${CYAN}=============================================================${NC}"
    log "XPlenum Regression Test Suite"
    log "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    log "Host: $(hostname)"
    log "Results: $RESULTS_DIR"
    log "${CYAN}=============================================================${NC}"

    log "\nTool availability:"
    check_tool iverilog || true
    check_tool vvp || true
    check_tool sby || true
    check_tool yosys || true
    check_tool verilator || true

    local mode="${1:---all}"

    case "$mode" in
        --standalone)  run_standalone ;;
        --integration) run_integration ;;
        --formal)      run_formal ;;
        --all)
            run_standalone
            run_integration
            run_formal
            ;;
        *)
            log "Usage: $0 [--standalone|--integration|--formal|--all]"
            exit 1
            ;;
    esac

    log "\n${CYAN}=============================================================${NC}"
    log "Regression Summary"
    log "${CYAN}=============================================================${NC}"
    log "  ${GREEN}Passed: $TOTAL_PASS${NC}"
    log "  ${RED}Failed: $TOTAL_FAIL${NC}"
    log "  ${YELLOW}Skipped: $TOTAL_SKIP${NC}"

    if [[ $TOTAL_FAIL -gt 0 ]]; then
        log "\n${RED}REGRESSION FAILED${NC}"
        exit 1
    else
        log "\n${GREEN}REGRESSION PASSED${NC}"
        exit 0
    fi
}

main "$@"
