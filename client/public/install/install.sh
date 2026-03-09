#!/usr/bin/env bash
# Salvi Framework / PlenumNET Installer
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Patent(s) Pending - All Rights Reserved
#
# Usage: curl -fsSL https://plenumnet.replit.app/install/install.sh | bash

set -euo pipefail

VERSION="3.0.0"
REPO_URL="https://github.com/SigmaWolf-8/Ternary"
INSTALL_DIR="${HOME}/SalviFramework"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
DIM='\033[2m'
NC='\033[0m'

banner() {
    echo ""
    echo -e "${CYAN}  =========================================${NC}"
    echo -e "${CYAN}    Salvi Framework Installer v${VERSION}${NC}"
    echo -e "${CYAN}    PlenumNET - Post-Quantum Infrastructure${NC}"
    echo -e "${CYAN}  =========================================${NC}"
    echo ""
    echo -e "${DIM}  Copyright 2025-2026 Capomastro Holdings Ltd.${NC}"
    echo ""
}

check_cmd() {
    local cmd=$1 name=$2 url=$3
    if command -v "$cmd" &>/dev/null; then
        local ver
        ver=$("$cmd" --version 2>&1 | head -1)
        echo -e "  ${GREEN}[OK]${NC} $name found: ${DIM}$ver${NC}"
        return 0
    else
        echo -e "  ${YELLOW}[--]${NC} $name not found"
        echo -e "       Install from: ${DIM}$url${NC}"
        return 1
    fi
}

main() {
    banner

    echo "  Checking prerequisites..."
    echo ""

    local has_git=true has_cargo=true
    check_cmd git "Git" "https://git-scm.com" || has_git=false
    check_cmd cargo "Rust/Cargo" "https://rustup.rs" || has_cargo=false
    echo ""

    if [ "$has_git" = false ]; then
        echo -e "  ${RED}Git is required. Install it first, then re-run this installer.${NC}"
        exit 1
    fi

    echo -e "  Install location: ${INSTALL_DIR}"
    echo ""

    if [ -d "$INSTALL_DIR" ]; then
        echo -e "  ${YELLOW}Directory exists. Updating...${NC}"
        cd "$INSTALL_DIR"
        git pull origin main 2>&1 | sed 's/^/    /'
    else
        echo "  Step 1/3: Cloning repository..."
        git clone "$REPO_URL" "$INSTALL_DIR" 2>&1 | sed 's/^/    /'
        cd "$INSTALL_DIR"
    fi

    echo ""
    echo "  Step 2/3: Checking Rust toolchain..."

    if [ "$has_cargo" = true ]; then
        echo "  Step 3/3: Building framework (this may take a few minutes)..."
        echo ""
        set +e
        cargo build --release 2>&1 | grep -E "Compiling|Finished|Downloaded|error" | sed 's/^/    /'
        BUILD_EXIT=$?
        set -e
        echo ""
        if [ $BUILD_EXIT -eq 0 ]; then
            echo -e "  ${GREEN}Build successful!${NC}"
        else
            echo -e "  ${YELLOW}Build had errors. The source code is still available at:${NC}"
            echo "  $INSTALL_DIR"
        fi
    else
        echo ""
        echo -e "  ${YELLOW}Rust is not installed. Source code downloaded to:${NC}"
        echo "  $INSTALL_DIR"
        echo ""
        echo "  To build later:"
        echo -e "    ${DIM}1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        echo -e "    ${DIM}2. Open a new terminal${NC}"
        echo -e "    ${DIM}3. Run: cd $INSTALL_DIR && cargo build --release${NC}"
    fi

    echo ""
    echo -e "${CYAN}  =========================================${NC}"
    echo -e "  ${GREEN}  Installation Complete${NC}"
    echo -e "${CYAN}  =========================================${NC}"
    echo ""
    echo "  Location:    $INSTALL_DIR"
    echo "  Version:     v$VERSION"
    echo "  Docs:        https://plenumnet.replit.app/docs"
    echo "  GitHub:      $REPO_URL"
    echo ""
    echo "  Quick start:"
    echo -e "    ${DIM}cd $INSTALL_DIR${NC}"
    echo -e "    ${DIM}cargo test        # Run 2,276 tests${NC}"
    echo -e "    ${DIM}cargo bench       # Run benchmarks${NC}"
    echo ""
}

main "$@"
