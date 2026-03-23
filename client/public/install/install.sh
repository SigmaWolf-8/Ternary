#!/usr/bin/env bash
# PlenumNET / Salvi Framework Installer
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Patent(s) Pending - All Rights Reserved
#
# Usage: curl -fsSL https://plenumnet.replit.app/install/install.sh | bash

set -euo pipefail

VERSION="2.4.0"
REPO_URL="https://github.com/SigmaWolf-8/Ternary"
INSTALL_DIR="${HOME}/PlenumNET"
IDENTITY_BASE="${HOME}/.plenumnet"
export CARGO_BUILD_JOBS=1

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
DIM='\033[2m'
NC='\033[0m'

banner() {
    echo ""
    echo -e "${CYAN}  ========================================================${NC}"
    echo -e "${CYAN}    PlenumNET Installer v${VERSION}${NC}"
    echo -e "${CYAN}    Salvi Framework - Post-Quantum Internet Infrastructure${NC}"
    echo -e "${DIM}    Capomastro Holdings Ltd.${NC}"
    echo -e "${CYAN}  ========================================================${NC}"
    echo ""
    echo -e "  Install location: ${INSTALL_DIR}"
    echo ""
}

check_cmd() {
    local cmd=$1 name=$2 url=$3
    if command -v "$cmd" &>/dev/null; then
        local ver
        ver=$("$cmd" --version 2>&1 | head -1)
        echo -e "    ${GREEN}[OK]${NC} $name : ${DIM}$ver${NC}"
        return 0
    else
        echo -e "    ${YELLOW}[--]${NC} $name : not installed"
        echo -e "         Get it from: ${DIM}$url${NC}"
        return 1
    fi
}

main() {
    banner

    echo "  Step 1 of 4: Checking prerequisites"
    echo -e "  ${DIM}-----------------------------------${NC}"

    local has_git=true has_cargo=true
    check_cmd git "Git" "https://git-scm.com" || has_git=false
    check_cmd cargo "Rust/Cargo" "https://rustup.rs" || has_cargo=false
    echo ""

    if [ "$has_git" = false ]; then
        echo -e "  ${RED}ERROR: Git is required but not installed.${NC}"
        echo "  Install Git first, then re-run this installer."
        exit 1
    fi

    echo "  Step 2 of 4: Downloading PlenumNET"
    echo -e "  ${DIM}-----------------------------------${NC}"

    if [ -d "$INSTALL_DIR/.git" ]; then
        echo -e "    ${YELLOW}Found existing installation. Updating...${NC}"
        cd "$INSTALL_DIR"
        git pull --ff-only origin main 2>&1 | sed 's/^/    /'
    elif [ -d "$INSTALL_DIR" ]; then
        echo -e "    ${YELLOW}Found non-git installation. Re-cloning...${NC}"
        rm -rf "$INSTALL_DIR"
        git clone --depth 1 "$REPO_URL" "$INSTALL_DIR" 2>&1 | sed 's/^/    /'
        cd "$INSTALL_DIR"
    else
        echo "    Cloning PlenumNET repository..."
        git clone --depth 1 "$REPO_URL" "$INSTALL_DIR" 2>&1 | sed 's/^/    /'
        cd "$INSTALL_DIR"
    fi

    echo ""
    echo "  Step 3 of 4: Building inter-cube daemon"
    echo -e "  ${DIM}-----------------------------------${NC}"

    if [ "$has_cargo" = true ]; then
        echo "    Building inter-cube daemon (CARGO_BUILD_JOBS=1)..."
        echo ""
        set +e
        cargo build --release -p inter-cube 2>&1 | grep -E "Compiling|Finished|Downloaded|error" | sed 's/^/    /'
        BUILD_EXIT=$?
        set -e
        echo ""
        if [ $BUILD_EXIT -eq 0 ]; then
            echo -e "    ${GREEN}Build successful!${NC}"
        else
            echo -e "    ${YELLOW}Build had errors (source code is still available).${NC}"
            echo "    You can retry: cd $INSTALL_DIR && CARGO_BUILD_JOBS=1 cargo build --release -p inter-cube"
        fi
    else
        echo -e "    ${YELLOW}Skipping build (Rust not installed).${NC}"
        echo ""
        echo "    To build later:"
        echo -e "      ${DIM}1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        echo -e "      ${DIM}2. Open a new terminal${NC}"
        echo -e "      ${DIM}3. Run: cd $INSTALL_DIR && CARGO_BUILD_JOBS=1 cargo build --release -p inter-cube${NC}"
    fi

    echo ""
    echo "  Step 4 of 4: Generating daemon identities"
    echo -e "  ${DIM}-----------------------------------${NC}"

    DAEMON_EXE="$INSTALL_DIR/target/release/inter-cube-daemon"
    if [ -x "$DAEMON_EXE" ]; then
        NEXT_ID=1
        while [ -f "$IDENTITY_BASE/identity-$NEXT_ID/master.key" ]; do
            NEXT_ID=$((NEXT_ID + 1))
        done
        AGENT_DIR="$IDENTITY_BASE/identity-$NEXT_ID"
        mkdir -p "$AGENT_DIR"
        echo "    Generating identity #$NEXT_ID..."
        CUBE_MODE=keygen CUBE_IDENTITY_DIR="$AGENT_DIR" "$DAEMON_EXE" 2>/dev/null || true
        if [ -f "$AGENT_DIR/master.key" ]; then
            echo -e "    ${GREEN}Daemon #$NEXT_ID identity created.${NC}"
        else
            echo -e "    ${YELLOW}WARNING: Identity #$NEXT_ID key generation may have failed.${NC}"
        fi
        ENGINE_PORT=$((8080 + (NEXT_ID - 1) * 2))
        DAEMON_PORT=$((ENGINE_PORT + 1))
        echo ""
        echo -e "    ${DIM}To start daemon #$NEXT_ID:${NC}"
        echo -e "    ${DIM}CUBE_MODE=cube CUBE_API_PORT=$DAEMON_PORT LLM_PORT=$ENGINE_PORT \\${NC}"
        echo -e "    ${DIM}CUBE_CRS_URL=https://plenumnet.replit.app \\${NC}"
        echo -e "    ${DIM}CUBE_IDENTITY_DIR=$AGENT_DIR $DAEMON_EXE${NC}"
        echo ""
        echo -e "    Run this installer again to add another daemon."
    else
        echo -e "    ${YELLOW}Daemon binary not found. Skipping identity generation.${NC}"
    fi

    echo ""
    echo -e "${CYAN}  ========================================================${NC}"
    echo -e "    ${GREEN}PlenumNET Installation Complete${NC}"
    echo -e "${CYAN}  ========================================================${NC}"
    echo ""
    echo "  Installed to:  $INSTALL_DIR"
    echo "  Version:       v$VERSION"
    echo "  Documentation: https://plenumnet.replit.app/docs"
    echo "  GitHub:        $REPO_URL"
    echo ""
    echo "  What's inside:"
    echo -e "    ${DIM}$INSTALL_DIR/src/kernel/     - Ternary kernel + crypto (Rust)${NC}"
    echo -e "    ${DIM}$INSTALL_DIR/ternary-math/   - Math library${NC}"
    echo -e "    ${DIM}$INSTALL_DIR/shared/          - TypeScript shared modules${NC}"
    echo -e "    ${DIM}$INSTALL_DIR/services/        - TDNS, Inter-Cube services${NC}"
    echo ""
    echo "  Next steps:"
    echo -e "    ${DIM}cd $INSTALL_DIR${NC}"
    echo -e "    ${DIM}cargo test          # Run tests${NC}"
    echo -e "    ${DIM}# Run installer again to add more daemons${NC}"
    echo ""
}

main "$@"
