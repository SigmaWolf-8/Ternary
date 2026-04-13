#!/bin/bash
set -e
echo "[deploy-clean] Removing non-runtime directories..."
for dir in target .git .github .cache .local .canvas .pythonlibs .agents artifacts attached_assets src ternary-math libternary libternary-improvements XPlenum ninja-exec plenumlan sign-here benchmarks contracts cli tools docs salvi_docs references kong keys github-push tests installer-output plenumnet-data assets deployments; do
  if [ -d "$dir" ]; then
    rm -rf "$dir"
    echo "  removed $dir"
  fi
done
if [ -d ".local/state" ]; then rm -rf .local/state; echo "  removed .local/state"; fi
if [ -d ".local/share" ]; then rm -rf .local/share; echo "  removed .local/share"; fi
echo "[deploy-clean] Removing non-runtime files..."
find . -maxdepth 1 -name "*.md" -o -name "*.txt" -o -name "*.log" -o -name "Cargo.toml" -o -name "Cargo.lock" -o -name "docker-compose*.yml" | xargs rm -f 2>/dev/null || true
rm -f .gitignore .gitattributes Dockerfile .dockerignore 2>/dev/null || true
echo "[deploy-clean] Stripping native addon symbols..."
if [ -f "server/crypto/sponge-native.node" ]; then
  strip --strip-debug server/crypto/sponge-native.node 2>/dev/null && echo "  stripped sponge-native.node" || echo "  strip not available (ok)"
fi
echo "[deploy-clean] Done."
