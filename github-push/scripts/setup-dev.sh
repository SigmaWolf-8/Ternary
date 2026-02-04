#!/bin/bash
# Post-Quantum Ternary Internet Development Environment Setup Script
# Version: 1.0.0

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    log_error "This script should not be run as root"
    exit 1
fi

# Check OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM=linux;;
    Darwin*)    PLATFORM=macos;;
    CYGWIN*)    PLATFORM=windows;;
    MINGW*)     PLATFORM=windows;;
    *)          PLATFORM="unknown"
esac

log_info "Detected platform: $PLATFORM"

# Create required directories
log_info "Creating directory structure..."
mkdir -p keys/signing
mkdir -p keys/encryption
mkdir -p config
mkdir -p scripts
mkdir -p logs
mkdir -p tmp

# Check for required tools
check_tool() {
    if command -v "$1" >/dev/null 2>&1; then
        log_success "$1 is installed"
        return 0
    else
        log_warning "$1 is not installed"
        return 1
    fi
}

log_info "Checking for required tools..."

REQUIRED_TOOLS=("curl" "git" "make")
MISSING_TOOLS=()

for tool in "${REQUIRED_TOOLS[@]}"; do
    if ! check_tool "$tool"; then
        MISSING_TOOLS+=("$tool")
    fi
done

if [ ${#MISSING_TOOLS[@]} -ne 0 ]; then
    log_warning "Missing tools: ${MISSING_TOOLS[*]}"
    log_info "Some tools are missing but setup will continue"
fi

# Install Rust if not present
if ! command -v rustc >/dev/null 2>&1; then
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    log_success "Rust installed"
else
    log_success "Rust is already installed ($(rustc --version))"
fi

# Install Rust components
log_info "Installing Rust components..."
rustup component add rustfmt 2>/dev/null || true
rustup component add clippy 2>/dev/null || true

# Install cargo tools
log_info "Installing cargo tools..."
cargo install cargo-audit 2>/dev/null || log_warning "cargo-audit already installed or failed"
cargo install cargo-watch 2>/dev/null || log_warning "cargo-watch already installed or failed"

# Clone submodules if any
if [ -f ".gitmodules" ]; then
    log_info "Updating git submodules..."
    git submodule update --init --recursive
fi

# Install project dependencies
log_info "Installing project dependencies..."
if [ -d "src/kernel" ]; then
    cd src/kernel && cargo fetch 2>/dev/null || true && cd ../..
fi
if [ -d "src/tsl" ]; then
    cd src/tsl && cargo fetch 2>/dev/null || true && cd ../..
fi
if [ -d "src/libternary" ]; then
    cd src/libternary && cargo fetch 2>/dev/null || true && cd ../..
fi
if [ -d "src/timing-api" ]; then
    cd src/timing-api && cargo fetch 2>/dev/null || true && cd ../..
fi

log_success "Development environment setup complete!"
log_info ""
log_info "Next steps:"
log_info "  1. Run 'make build' to build all components"
log_info "  2. Run 'make test' to run all tests"
log_info "  3. Run 'make help' for more options"
