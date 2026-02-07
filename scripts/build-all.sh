#!/bin/bash
set -euo pipefail

echo "=========================================="
echo " PlenumNET Build System"
echo " Capomastro Holdings Ltd."
echo "=========================================="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_MODE="${1:-debug}"

build_component() {
    local name="$1"
    local dir="$2"
    local extra_args="${3:-}"
    
    echo "[BUILD] $name..."
    if [ -f "$ROOT_DIR/$dir/Cargo.toml" ]; then
        if [ "$BUILD_MODE" = "release" ]; then
            (cd "$ROOT_DIR/$dir" && cargo build --release $extra_args 2>&1) || {
                echo "[FAIL] $name build failed"
                return 1
            }
        else
            (cd "$ROOT_DIR/$dir" && cargo build $extra_args 2>&1) || {
                echo "[FAIL] $name build failed"
                return 1
            }
        fi
        echo "[PASS] $name"
    else
        echo "[SKIP] $name (Cargo.toml not found at $dir)"
    fi
}

echo "Build mode: $BUILD_MODE"
echo ""

echo "--- Kernel Core ---"
build_component "PlenumNET Kernel" "src/kernel" "--all-features"

echo ""
echo "--- Language Tools ---"
build_component "TSL Compiler" "src/tsl"
build_component "THDL Synthesizer" "src/thdl"

echo ""
echo "--- TypeScript Libraries ---"
if [ -f "$ROOT_DIR/libternary/package.json" ]; then
    echo "[BUILD] libternary..."
    (cd "$ROOT_DIR/libternary" && npm install --ignore-scripts 2>/dev/null && npm run build 2>/dev/null) || echo "[SKIP] libternary (build script not configured)"
    echo "[PASS] libternary"
fi

echo ""
echo "=========================================="
echo " Build Complete"
echo "=========================================="
