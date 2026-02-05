#!/bin/bash
# Build Algorand Smart Contracts
# Requires: Python 3.10+, pyteal

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"

echo "Building Algorand Smart Contracts..."

# Create build directory
mkdir -p "$BUILD_DIR"

# Check if pyteal is installed
if ! python3 -c "import pyteal" 2>/dev/null; then
    echo "Installing PyTEAL..."
    pip3 install pyteal
fi

# Build TAT-GOV-001: Governance Contract
echo "Building TAT-GOV-001: Ternary Governance Contract..."
python3 "$SCRIPT_DIR/ternary-governance-contract.py" > "$BUILD_DIR/tat-gov-001.teal"

# Build TAT-DIST-001: Reward Distributor
echo "Building TAT-DIST-001: Ternary Reward Distributor..."
python3 "$SCRIPT_DIR/ternary-reward-distributor.py" > "$BUILD_DIR/tat-dist-001.teal"

echo ""
echo "Build complete! Contracts output to: $BUILD_DIR"
echo "  - tat-gov-001.teal (Governance)"
echo "  - tat-dist-001.teal (Reward Distributor)"
