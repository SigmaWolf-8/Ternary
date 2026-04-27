#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# capture-shim-baseline.sh
#
# Capture pre-shim test pass/fail/ignored baselines for the two
# crates that the `aasc` (Algeometric Arc Σ-182 Calculi)
# consolidation is targeted to reshape:
#
#   - plenumnet-kernel   (src/kernel)
#   - ternary-math       (ternary-math)
#
# Output:
#   .baseline/<crate>-<scope>.txt        — full cargo test output
#   .baseline/baseline-summary.json      — machine-readable counts
#
# Scopes:
#   shim-gate  — fast lib unit tests only (used as the PR gate)
#   full       — lib + integration tests (used as the merge gate)
#
# Exit status mirrors the worst cargo test exit code so this script
# can be wired straight into CI without further parsing.
#
# Capomastro Holdings Ltd. — Applied Physics Division
# ──────────────────────────────────────────────────────────────────
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
OUT_DIR="$ROOT_DIR/.baseline"
mkdir -p "$OUT_DIR"

SCOPE="${1:-shim-gate}"
case "$SCOPE" in
    shim-gate|full) ;;
    *)
        echo "usage: $0 [shim-gate|full]" >&2
        exit 64
        ;;
esac

cd "$ROOT_DIR"

WORST_RC=0
SUMMARY_JSON="$OUT_DIR/baseline-summary.json"
echo "{" > "$SUMMARY_JSON"
echo "  \"scope\": \"$SCOPE\"," >> "$SUMMARY_JSON"
echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$SUMMARY_JSON"
echo "  \"crates\": [" >> "$SUMMARY_JSON"

FIRST_CRATE=1

run_crate() {
    local crate_name="$1"
    local label="$2"
    shift 2
    local cargo_args=("$@")

    local log="$OUT_DIR/${crate_name}-${SCOPE}.txt"

    echo "──────────────────────────────────────────────────────"
    echo " $label  ($crate_name, scope=$SCOPE)"
    echo "──────────────────────────────────────────────────────"

    set +e
    cargo test -p "$crate_name" "${cargo_args[@]}" 2>&1 | tee "$log"
    local rc=${PIPESTATUS[0]}
    set -e
    if [ "$rc" -gt "$WORST_RC" ]; then
        WORST_RC=$rc
    fi

    # Aggregate every "test result: ..." line cargo test emits
    # (one per binary). Sums counts across binaries.
    local passed failed ignored
    passed=$(grep -E "^test result:" "$log" \
             | sed -E 's/.* ([0-9]+) passed.*/\1/' \
             | awk '{s+=$1} END {print s+0}')
    failed=$(grep -E "^test result:" "$log" \
             | sed -E 's/.* ([0-9]+) failed.*/\1/' \
             | awk '{s+=$1} END {print s+0}')
    ignored=$(grep -E "^test result:" "$log" \
              | sed -E 's/.* ([0-9]+) ignored.*/\1/' \
              | awk '{s+=$1} END {print s+0}')

    echo
    echo "[$crate_name] passed=$passed failed=$failed ignored=$ignored rc=$rc"
    echo

    if [ "$FIRST_CRATE" -eq 0 ]; then
        echo "    ," >> "$SUMMARY_JSON"
    fi
    FIRST_CRATE=0
    {
        echo "    {"
        echo "      \"crate\": \"$crate_name\","
        echo "      \"label\": \"$label\","
        echo "      \"passed\": $passed,"
        echo "      \"failed\": $failed,"
        echo "      \"ignored\": $ignored,"
        echo "      \"exit_code\": $rc,"
        echo "      \"log\": \".baseline/${crate_name}-${SCOPE}.txt\""
        echo "    }"
    } >> "$SUMMARY_JSON"
}

if [ "$SCOPE" = "shim-gate" ]; then
    # Fast PR gate: lib unit tests only, no integration binaries.
    run_crate plenumnet-kernel "Kernel · lib unit tests"   --lib
    run_crate ternary-math    "Ternary-math · lib unit tests" --lib
else
    # Full baseline: literal `cargo test -p <crate>` per crate so that
    # lib unittests, every bin unittest, every integration target
    # AND doctests are all covered. Numbers are aggregated across
    # every "test result:" line cargo emits per crate.
    #
    # Doctest failures in `plenumnet-kernel` are expected at this
    # baseline (pre-existing; tracked in
    # docs/audit/bare-metal-incorporation.md). The non-zero exit
    # code is propagated as `worst_rc` so a future shim PR can prove
    # it does not introduce *new* failures, while the recorded
    # passed/failed counts stay diffable.
    run_crate plenumnet-kernel "Kernel · cargo test -p plenumnet-kernel"
    run_crate ternary-math    "Ternary-math · cargo test -p ternary-math"
fi

echo "  ]" >> "$SUMMARY_JSON"
echo "}" >> "$SUMMARY_JSON"

echo
echo "Baseline summary written to: $SUMMARY_JSON"
echo "Per-crate logs in: $OUT_DIR/"
exit "$WORST_RC"
