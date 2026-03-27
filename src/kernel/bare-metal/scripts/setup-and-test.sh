#!/usr/bin/env bash
# PlenumNET Ternary Kernel — One-Click Build & QEMU Test
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.
#
# Installs all dependencies, clones the repo, builds the kernel, and
# runs the QEMU bare-metal boot test. Idempotent — safe to run repeatedly.
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
BOLD='\033[1m'
NC='\033[0m'

log()  { echo -e "${CYAN}[SETUP]${NC} $*"; }
ok()   { echo -e "${GREEN}[  OK ]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }

echo -e "${BOLD}"
echo "================================================================"
echo "  PlenumNET Bare-Metal Kernel — Setup & Test"
echo "  Capomastro Holdings Ltd. — Applied Physics Division"
echo "================================================================"
echo -e "${NC}"

if [[ "$(uname -s)" == *MINGW* ]] || [[ "$(uname -s)" == *MSYS* ]]; then
    fail "Run this inside WSL or native Linux, not Windows PowerShell/CMD."
fi

REPO_DIR="${PLENUM_REPO:-$HOME/Ternary}"

log "Step 1/5: System packages (QEMU + build tools)..."
if command -v apt-get &>/dev/null; then
    sudo apt-get update -qq 2>/dev/null || true
    sudo apt-get install -y -qq --fix-missing \
        qemu-system-x86 build-essential 2>/dev/null || {
        fail "Could not install system packages. Check your package sources."
    }
elif command -v dnf &>/dev/null; then
    sudo dnf install -y qemu-system-x86 gcc make 2>/dev/null
elif command -v pacman &>/dev/null; then
    sudo pacman -S --noconfirm qemu-full base-devel 2>/dev/null
else
    echo "[WARN] Unknown package manager — ensure qemu-system-x86_64 and build-essential are installed."
fi
ok "System packages ready"

log "Step 2/5: Rust toolchain..."
if ! command -v rustc &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
    source "$HOME/.cargo/env"
    ok "Rust installed (nightly)"
else
    source "$HOME/.cargo/env" 2>/dev/null || true
    ok "Rust present: $(rustc --version)"
fi

log "Step 3/5: Nightly toolchain + rust-src..."
rustup toolchain install nightly --component rust-src 2>/dev/null || true
ok "Nightly + rust-src ready"

log "Step 4/5: Repository..."
if [ -d "$REPO_DIR/.git" ]; then
    log "Pulling latest into $REPO_DIR..."
    (cd "$REPO_DIR" && git pull --ff-only 2>/dev/null) || true
else
    git clone https://github.com/SigmaWolf-8/Ternary.git "$REPO_DIR"
fi
ok "Repository at $REPO_DIR"

log "Step 5/5: Build and test..."
cd "$REPO_DIR/src/kernel/bare-metal"

bash scripts/build.sh

BINARY="target/x86_64-unknown-none/debug/ternary-kernel"
if [ ! -f "$BINARY" ]; then
    fail "Kernel binary not found at $BINARY"
fi

ok "Kernel built — launching QEMU..."
echo ""
bash scripts/qemu-run.sh "$BINARY"
