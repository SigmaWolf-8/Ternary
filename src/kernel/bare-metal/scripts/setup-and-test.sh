#!/usr/bin/env bash
# PlenumNET Ternary Kernel — One-Click Build & QEMU Test
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.
#
# Usage (from any directory):
#   curl -sSf https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/src/kernel/bare-metal/scripts/setup-and-test.sh | bash
#
# Or locally:
#   bash src/kernel/bare-metal/scripts/setup-and-test.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[SETUP]${NC} $*"; }
ok()   { echo -e "${GREEN}[  OK ]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }

log "PlenumNET Bare-Metal Kernel — Setup & Test"
echo ""

if [[ "$(uname -s)" == *MINGW* ]] || [[ "$(uname -s)" == *MSYS* ]]; then
    fail "Run this inside WSL, not Windows PowerShell/CMD."
fi

log "Step 1/6: Updating package index..."
sudo apt-get update -qq 2>/dev/null || true

log "Step 2/6: Installing QEMU and build tools..."
sudo apt-get install -y -qq qemu-system-x86 qemu-system-arm qemu-system-misc build-essential 2>/dev/null || {
    log "Some packages failed — retrying with --fix-missing..."
    sudo apt-get install -y -qq --fix-missing qemu-system-x86 qemu-system-arm qemu-system-misc build-essential 2>/dev/null
}
ok "QEMU and build tools installed"

log "Step 3/6: Installing Rust (if needed)..."
if ! command -v rustc &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    ok "Rust installed"
else
    source "$HOME/.cargo/env" 2>/dev/null || true
    ok "Rust already installed: $(rustc --version)"
fi

log "Step 4/6: Installing nightly toolchain + bare-metal targets..."
rustup toolchain install nightly --component rust-src 2>/dev/null
rustup target add x86_64-unknown-none aarch64-unknown-none-softfloat riscv64gc-unknown-none-elf 2>/dev/null
ok "Rust nightly + bare-metal targets ready"

log "Step 5/6: Cloning repository..."
REPO_DIR="$HOME/Ternary"
if [ -d "$REPO_DIR/.git" ]; then
    log "Repository exists — pulling latest..."
    cd "$REPO_DIR" && git pull --ff-only 2>/dev/null || true
else
    git clone https://github.com/SigmaWolf-8/Ternary.git "$REPO_DIR"
fi
ok "Repository ready at $REPO_DIR"

log "Step 6/6: Building and testing kernel..."
cd "$REPO_DIR/src/kernel/bare-metal"
rustup override set nightly 2>/dev/null

bash scripts/build.sh

BINARY="target/x86_64-unknown-none/debug/ternary-kernel"
if [ -f "$BINARY" ]; then
    ok "Kernel built successfully"
    echo ""
    bash scripts/qemu-run.sh "$BINARY"
else
    fail "Kernel binary not found at $BINARY"
fi
