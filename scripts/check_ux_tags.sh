#!/usr/bin/env bash
# check_ux_tags.sh — enforces L.G.O.UX{p}.{s1}[.{s2}...]__name.rs canonical naming
# for every module under the algeometric-arc-sigma182-calculi (aasc) crate.
#
# Exit codes:
#   0 = all files conform
#   1 = at least one file violates the convention

set -euo pipefail

ROOT="${1:-src/aasc}"
PATTERN='^[1-9]\.[1-9]\.[1-9]\.UX[1-9](\.[1-9])+__[a-z][a-z0-9_]*\.rs$'

if [[ ! -d "$ROOT" ]]; then
  echo "[check_ux_tags] root '$ROOT' not found — skipping (no aasc crate yet)"
  exit 0
fi

violations=0
total=0
while IFS= read -r -d '' f; do
  total=$((total+1))
  base="$(basename "$f")"
  # Allow lib.rs, mod.rs, build.rs, tests/, benches/ as exemptions
  case "$base" in
    lib.rs|mod.rs|build.rs|main.rs) continue ;;
  esac
  case "$f" in
    */tests/*|*/benches/*|*/examples/*) continue ;;
  esac
  if [[ ! "$base" =~ $PATTERN ]]; then
    echo "VIOLATION: $f  (basename '$base' does not match L.G.O.UX{p}.{s}__name.rs)"
    violations=$((violations+1))
  fi
done < <(find "$ROOT" -type f -name '*.rs' -print0)

if (( violations > 0 )); then
  echo ""
  echo "[check_ux_tags] FAIL — $violations of $total *.rs files do not carry a canonical L.G.O.UX address."
  echo "[check_ux_tags] See client/public/maps/aasc_canonical_map.png (UX Extension panel) for the digit table."
  exit 1
fi

echo "[check_ux_tags] OK — all $total *.rs files carry a canonical L.G.O.UX address."
exit 0
