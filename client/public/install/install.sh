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
        CRS_URL="https://plenumnet.replit.app"
        SERVICE_INSTALLED=false

        OS_TYPE="$(uname -s)"
        if [ "$OS_TYPE" = "Linux" ] && command -v systemctl &>/dev/null; then
            echo ""
            echo -e "    ${CYAN}Registering systemd service for daemon #$NEXT_ID...${NC}"
            sudo mkdir -p /etc/plenumnet /var/lib/plenumnet 2>/dev/null || true

            UNIT_SRC="$INSTALL_DIR/client/public/install/services/plenumnet-cube@.service"
            if [ -f "$UNIT_SRC" ] && [ ! -f /etc/systemd/system/plenumnet-cube@.service ]; then
                sudo cp "$UNIT_SRC" /etc/systemd/system/plenumnet-cube@.service 2>/dev/null || true
            fi

            if [ ! -f /usr/local/bin/inter-cube-daemon ] || [ "$DAEMON_EXE" -nt /usr/local/bin/inter-cube-daemon ]; then
                sudo cp "$DAEMON_EXE" /usr/local/bin/inter-cube-daemon 2>/dev/null || true
                sudo chmod 755 /usr/local/bin/inter-cube-daemon 2>/dev/null || true
            fi

            sudo tee "/etc/plenumnet/cube-${NEXT_ID}.env" > /dev/null 2>&1 <<ENVEOF || true
CUBE_MODE=cube
CUBE_API_PORT=$DAEMON_PORT
LLM_PORT=$ENGINE_PORT
CUBE_CRS_URL=$CRS_URL
RELAY_URL=$CRS_URL
CUBE_IDENTITY_DIR=$AGENT_DIR
CUBE_ROLE=inference
ENVEOF

            if sudo systemctl daemon-reload 2>/dev/null && \
               sudo systemctl enable "plenumnet-cube@${NEXT_ID}.service" 2>/dev/null && \
               sudo systemctl start "plenumnet-cube@${NEXT_ID}.service" 2>/dev/null; then
                SERVICE_INSTALLED=true
                echo -e "    ${GREEN}Daemon #$NEXT_ID registered as systemd service.${NC}"
            else
                echo -e "    ${YELLOW}Could not register systemd service (manual start available below).${NC}"
            fi

        elif [ "$OS_TYPE" = "Darwin" ]; then
            echo ""
            echo -e "    ${CYAN}Registering launchd service for daemon #$NEXT_ID...${NC}"
            LOG_DIR="$IDENTITY_BASE/logs"
            mkdir -p "$LOG_DIR" "${HOME}/Library/LaunchAgents" 2>/dev/null || true

            PLIST_PATH="${HOME}/Library/LaunchAgents/com.plenumnet.cube-${NEXT_ID}.plist"
            TEMPLATE="$INSTALL_DIR/client/public/install/services/com.plenumnet.cube.plist.template"

            if [ -f "$TEMPLATE" ]; then
                sed -e "s|__IDENTITY_ID__|$NEXT_ID|g" \
                    -e "s|__DAEMON_EXE__|$DAEMON_EXE|g" \
                    -e "s|__DAEMON_PORT__|$DAEMON_PORT|g" \
                    -e "s|__ENGINE_PORT__|$ENGINE_PORT|g" \
                    -e "s|__CRS_URL__|$CRS_URL|g" \
                    -e "s|__IDENTITY_DIR__|$AGENT_DIR|g" \
                    -e "s|__LOG_DIR__|$LOG_DIR|g" \
                    -e "s|__INSTALL_DIR__|$INSTALL_DIR|g" \
                    "$TEMPLATE" > "$PLIST_PATH"
            else
                cat > "$PLIST_PATH" <<PLISTEOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.plenumnet.cube-${NEXT_ID}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${DAEMON_EXE}</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>CUBE_MODE</key>
        <string>cube</string>
        <key>CUBE_API_PORT</key>
        <string>${DAEMON_PORT}</string>
        <key>LLM_PORT</key>
        <string>${ENGINE_PORT}</string>
        <key>CUBE_CRS_URL</key>
        <string>${CRS_URL}</string>
        <key>RELAY_URL</key>
        <string>${CRS_URL}</string>
        <key>CUBE_IDENTITY_DIR</key>
        <string>${AGENT_DIR}</string>
        <key>CUBE_ROLE</key>
        <string>inference</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>5</integer>
    <key>StandardOutPath</key>
    <string>${LOG_DIR}/cube-${NEXT_ID}-stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/cube-${NEXT_ID}-stderr.log</string>
    <key>WorkingDirectory</key>
    <string>${INSTALL_DIR}</string>
</dict>
</plist>
PLISTEOF
            fi

            launchctl unload "$PLIST_PATH" 2>/dev/null || true
            if launchctl load "$PLIST_PATH" 2>/dev/null; then
                SERVICE_INSTALLED=true
                echo -e "    ${GREEN}Daemon #$NEXT_ID registered as launchd service.${NC}"
            else
                echo -e "    ${YELLOW}Could not register launchd service (manual start available below).${NC}"
            fi
        fi

        echo ""
        if [ "$SERVICE_INSTALLED" = true ]; then
            echo -e "    ${GREEN}Daemon #$NEXT_ID is running as a system service and will auto-start on boot.${NC}"
        fi
        echo -e "    ${DIM}Manual start (fallback):${NC}"
        echo -e "    ${DIM}CUBE_MODE=cube CUBE_API_PORT=$DAEMON_PORT LLM_PORT=$ENGINE_PORT \\${NC}"
        echo -e "    ${DIM}CUBE_CRS_URL=$CRS_URL RELAY_URL=$CRS_URL \\${NC}"
        echo -e "    ${DIM}CUBE_IDENTITY_DIR=$AGENT_DIR $DAEMON_EXE${NC}"
        echo ""
        echo -e "    Run this installer again to add another daemon."
    else
        echo -e "    ${YELLOW}Daemon binary not found. Skipping identity generation.${NC}"
    fi

    SERVICE_MGR="$INSTALL_DIR/client/public/install/plenumnet-service.sh"
    if [ -f "$SERVICE_MGR" ]; then
        chmod +x "$SERVICE_MGR" 2>/dev/null || true
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
    if [ "${SERVICE_INSTALLED:-false}" = true ]; then
        echo -e "  ${GREEN}Daemon is running as a system service and will auto-start on boot.${NC}"
        echo ""
        echo "  Service management:"
        if [ "$(uname -s)" = "Linux" ]; then
            echo -e "    ${DIM}systemctl status plenumnet-cube@$NEXT_ID    # Check status${NC}"
            echo -e "    ${DIM}journalctl -u plenumnet-cube@$NEXT_ID -f    # View logs${NC}"
            echo -e "    ${DIM}systemctl restart plenumnet-cube@$NEXT_ID   # Restart${NC}"
        else
            echo -e "    ${DIM}launchctl list com.plenumnet.cube-$NEXT_ID  # Check status${NC}"
            echo -e "    ${DIM}tail -f ~/.plenumnet/logs/cube-$NEXT_ID-stdout.log  # View logs${NC}"
        fi
        echo ""
        echo "  Or use the service manager:"
        echo -e "    ${DIM}bash $SERVICE_MGR status             # All daemon statuses${NC}"
        echo -e "    ${DIM}bash $SERVICE_MGR logs $NEXT_ID           # View daemon logs${NC}"
        echo -e "    ${DIM}bash $SERVICE_MGR uninstall $NEXT_ID      # Remove service${NC}"
    else
        echo "  Next steps:"
        echo -e "    ${DIM}cd $INSTALL_DIR${NC}"
        echo -e "    ${DIM}cargo test          # Run tests${NC}"
        echo -e "    ${DIM}# Run installer again to add more daemons${NC}"
    fi
    echo ""
}

main "$@"
