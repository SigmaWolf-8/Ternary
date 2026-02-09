#!/bin/bash
# ================================================================
# Entropy Sample Collection Script
# Salvi Ternary Cryptographic Module v3.0.0
# Capomastro Holdings Ltd.
# ================================================================
#
# Collects 1M+ raw noise samples for SP 800-90B lab assessment.
# CSTL runs NIST SP 800-90B test suite (ea_iid / ea_non_iid)
# against this data to qualify the entropy source.
#
# Usage: ./scripts/collect-entropy-samples.sh [sample_count]
# Output: target/entropy-samples-YYYYMMDD.bin
# ================================================================

set -euo pipefail

SAMPLE_COUNT="${1:-1000000}"
OUTPUT="target/entropy-samples-$(date +%Y%m%d).bin"

echo "=== Entropy Sample Collection ==="
echo "Collecting $SAMPLE_COUNT entropy samples..."
echo "Output: $OUTPUT"
echo ""

mkdir -p target

cd src/kernel
cargo test --release entropy::tests::collect_raw_samples -- \
  --nocapture \
  --sample-count "$SAMPLE_COUNT" \
  --output-file "../../$OUTPUT" \
  2>&1 | tail -3

cd ../..

if [ -f "$OUTPUT" ]; then
  SIZE=$(stat --format=%s "$OUTPUT" 2>/dev/null || echo "0")
  echo ""
  echo "=== Collection Complete ==="
  echo "Samples: $SAMPLE_COUNT"
  echo "File:    $OUTPUT"
  echo "Size:    $SIZE bytes"
  echo ""
  echo "Submit this file to CSTL for SP 800-90B entropy assessment."
  echo "Lab will run: ea_iid $OUTPUT 8"
  echo "         and: ea_non_iid $OUTPUT 8"
else
  echo ""
  echo "[WARN] Output file not created."
  echo "The entropy collection test harness may need to be configured."
  echo "Ensure entropy::tests::collect_raw_samples is implemented."
fi
