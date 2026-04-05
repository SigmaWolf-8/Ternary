#!/bin/bash
set -e
echo "[deploy-clean] Removing non-runtime directories..."
for dir in target .cache artifacts attached_assets src ternary-math libternary libternary-improvements XPlenum ninja-exec plenumlan sign-here benchmarks contracts cli tools docs salvi_docs references kong keys github-push .github tests installer-output plenumnet-data scripts; do
  if [ -d "$dir" ]; then
    rm -rf "$dir"
    echo "  removed $dir"
  fi
done
if [ -d ".local/state" ]; then rm -rf .local/state; echo "  removed .local/state"; fi
if [ -d ".local/share" ]; then rm -rf .local/share; echo "  removed .local/share"; fi
echo "[deploy-clean] Done."
