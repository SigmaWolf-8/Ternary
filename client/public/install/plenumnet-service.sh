#!/usr/bin/env bash
# PlenumNET Service Manager — Linux (systemd) & macOS (launchd)
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Patent(s) Pending - All Rights Reserved

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
DIM='\033[2m'
NC='\033[0m'

INSTALL_DIR="${PLENUMNET_DIR:-${HOME}/PlenumNET}"
IDENTITY_BASE="${HOME}/.plenumnet"
LOG_DIR="${HOME}/.plenumnet/logs"

detect_platform() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       echo "unknown" ;;
    esac
}

PLATFORM="$(detect_platform)"

usage() {
    echo ""
    echo -e "${CYAN}PlenumNET Service Manager${NC}"
    echo ""
    echo "Usage: $0 <command> [identity-id]"
    echo ""
    echo "Commands:"
    echo "  status   [id]   Show service status (all or specific identity)"
    echo "  start    <id>   Start a daemon service"
    echo "  stop     <id>   Stop a daemon service"
    echo "  restart  <id>   Restart a daemon service"
    echo "  logs     <id>   View daemon logs"
    echo "  install  <id>   Register and start a daemon as a service"
    echo "  uninstall <id>  Stop and deregister a daemon service"
    echo ""
    echo "Examples:"
    echo "  $0 status           # Show all daemon statuses"
    echo "  $0 start 1          # Start daemon identity #1"
    echo "  $0 logs 1           # View logs for daemon #1"
    echo "  $0 uninstall 1      # Uninstall daemon #1 service"
    echo ""
    exit 1
}

service_name_linux() {
    echo "plenumnet-cube@${1}.service"
}

plist_label() {
    echo "com.plenumnet.cube-${1}"
}

plist_path() {
    echo "${HOME}/Library/LaunchAgents/com.plenumnet.cube-${1}.plist"
}

cmd_status() {
    local id="${1:-}"
    echo ""
    echo -e "${CYAN}PlenumNET Daemon Service Status${NC}"
    echo -e "${DIM}Platform: ${PLATFORM}${NC}"
    echo ""

    if [ "$PLATFORM" = "linux" ]; then
        if [ -n "$id" ]; then
            systemctl status "$(service_name_linux "$id")" 2>/dev/null || echo -e "  ${YELLOW}Service for identity #$id not found${NC}"
        else
            local found=false
            for unit_file in /etc/systemd/system/plenumnet-cube@*.service; do
                [ -e "$unit_file" ] || continue
                found=true
            done
            if [ "$found" = true ]; then
                systemctl list-units 'plenumnet-cube@*' --no-pager 2>/dev/null || true
            fi
            for dir in "$IDENTITY_BASE"/identity-*; do
                [ -d "$dir" ] || continue
                local num
                num=$(basename "$dir" | sed 's/identity-//')
                local svc
                svc="$(service_name_linux "$num")"
                if systemctl is-enabled "$svc" &>/dev/null; then
                    local state
                    state=$(systemctl is-active "$svc" 2>/dev/null || echo "unknown")
                    echo -e "  Identity #$num: ${state}"
                else
                    echo -e "  Identity #$num: ${DIM}not registered as service${NC}"
                fi
            done
        fi
    elif [ "$PLATFORM" = "macos" ]; then
        if [ -n "$id" ]; then
            launchctl list "$(plist_label "$id")" 2>/dev/null || echo -e "  ${YELLOW}Service for identity #$id not found${NC}"
        else
            for dir in "$IDENTITY_BASE"/identity-*; do
                [ -d "$dir" ] || continue
                local num
                num=$(basename "$dir" | sed 's/identity-//')
                local label
                label="$(plist_label "$num")"
                if launchctl list "$label" &>/dev/null; then
                    echo -e "  Identity #$num: ${GREEN}loaded${NC}"
                else
                    echo -e "  Identity #$num: ${DIM}not registered as service${NC}"
                fi
            done
        fi
    else
        echo -e "${RED}Unsupported platform: ${PLATFORM}${NC}"
        exit 1
    fi
    echo ""
}

cmd_start() {
    local id="$1"
    echo -e "  Starting daemon #$id..."
    if [ "$PLATFORM" = "linux" ]; then
        sudo systemctl start "$(service_name_linux "$id")"
        echo -e "  ${GREEN}Daemon #$id started.${NC}"
    elif [ "$PLATFORM" = "macos" ]; then
        local plist
        plist="$(plist_path "$id")"
        if [ ! -f "$plist" ]; then
            echo -e "  ${RED}Plist not found: $plist${NC}"
            echo "  Run '$0 install $id' first."
            exit 1
        fi
        launchctl load "$plist" 2>/dev/null || true
        launchctl start "$(plist_label "$id")"
        echo -e "  ${GREEN}Daemon #$id started.${NC}"
    fi
}

cmd_stop() {
    local id="$1"
    echo -e "  Stopping daemon #$id..."
    if [ "$PLATFORM" = "linux" ]; then
        sudo systemctl stop "$(service_name_linux "$id")"
        echo -e "  ${GREEN}Daemon #$id stopped.${NC}"
    elif [ "$PLATFORM" = "macos" ]; then
        launchctl stop "$(plist_label "$id")" 2>/dev/null || true
        echo -e "  ${GREEN}Daemon #$id stopped.${NC}"
    fi
}

cmd_restart() {
    local id="$1"
    echo -e "  Restarting daemon #$id..."
    if [ "$PLATFORM" = "linux" ]; then
        sudo systemctl restart "$(service_name_linux "$id")"
        echo -e "  ${GREEN}Daemon #$id restarted.${NC}"
    elif [ "$PLATFORM" = "macos" ]; then
        launchctl stop "$(plist_label "$id")" 2>/dev/null || true
        sleep 1
        launchctl start "$(plist_label "$id")"
        echo -e "  ${GREEN}Daemon #$id restarted.${NC}"
    fi
}

cmd_logs() {
    local id="$1"
    if [ "$PLATFORM" = "linux" ]; then
        journalctl -u "$(service_name_linux "$id")" -f --no-pager
    elif [ "$PLATFORM" = "macos" ]; then
        local stdout_log="$LOG_DIR/cube-${id}-stdout.log"
        local stderr_log="$LOG_DIR/cube-${id}-stderr.log"
        if [ -f "$stdout_log" ]; then
            echo -e "${DIM}=== stdout ===${NC}"
            tail -f "$stdout_log" &
        fi
        if [ -f "$stderr_log" ]; then
            echo -e "${DIM}=== stderr ===${NC}"
            tail -f "$stderr_log" &
        fi
        wait
    fi
}

cmd_install() {
    local id="$1"
    local agent_dir="$IDENTITY_BASE/identity-$id"
    local daemon_exe="$INSTALL_DIR/target/release/inter-cube-daemon"

    if [ ! -x "$daemon_exe" ]; then
        echo -e "  ${RED}Daemon binary not found at $daemon_exe${NC}"
        echo "  Build first: cd $INSTALL_DIR && cargo build --release -p inter-cube"
        exit 1
    fi

    if [ ! -d "$agent_dir" ]; then
        echo -e "  ${RED}Identity directory not found: $agent_dir${NC}"
        echo "  Run the installer first to generate identities."
        exit 1
    fi

    local engine_port=$((8080 + (id - 1) * 2))
    local daemon_port=$((engine_port + 1))
    local crs_url="https://plenumnet.replit.app"

    if [ "$PLATFORM" = "linux" ]; then
        echo -e "  Installing systemd service for daemon #$id..."

        local env_dir="/etc/plenumnet"
        sudo mkdir -p "$env_dir" /var/lib/plenumnet

        sudo tee "$env_dir/cube-${id}.env" > /dev/null <<ENVEOF
CUBE_MODE=cube
CUBE_API_PORT=$daemon_port
LLM_PORT=$engine_port
CUBE_CRS_URL=$crs_url
RELAY_URL=$crs_url
CUBE_IDENTITY_DIR=$agent_dir
CUBE_ROLE=inference
ENVEOF

        local unit_src="$INSTALL_DIR/client/public/install/services/plenumnet-cube@.service"
        if [ -f "$unit_src" ]; then
            sudo cp "$unit_src" /etc/systemd/system/plenumnet-cube@.service
        else
            sudo tee /etc/systemd/system/plenumnet-cube@.service > /dev/null <<'UNITEOF'
[Unit]
Description=PlenumNET Inter-Cube Daemon (Identity %i)
Documentation=https://plenumnet.replit.app/docs
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=-/etc/plenumnet/cube-%i.env
ExecStart=/usr/local/bin/inter-cube-daemon
Restart=on-failure
RestartSec=5
StartLimitIntervalSec=300
StartLimitBurst=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=plenumnet-cube-%i
LimitNOFILE=65536
WorkingDirectory=/var/lib/plenumnet

[Install]
WantedBy=multi-user.target
UNITEOF
        fi

        if [ ! -f /usr/local/bin/inter-cube-daemon ] || [ "$daemon_exe" -nt /usr/local/bin/inter-cube-daemon ]; then
            sudo cp "$daemon_exe" /usr/local/bin/inter-cube-daemon
            sudo chmod 755 /usr/local/bin/inter-cube-daemon
        fi

        sudo systemctl daemon-reload
        sudo systemctl enable "plenumnet-cube@${id}.service"
        sudo systemctl start "plenumnet-cube@${id}.service"
        echo -e "  ${GREEN}Daemon #$id registered and started as systemd service.${NC}"
        echo -e "  ${DIM}Check status: systemctl status plenumnet-cube@${id}${NC}"
        echo -e "  ${DIM}View logs:    journalctl -u plenumnet-cube@${id} -f${NC}"

    elif [ "$PLATFORM" = "macos" ]; then
        echo -e "  Installing launchd service for daemon #$id..."

        mkdir -p "$LOG_DIR"
        mkdir -p "${HOME}/Library/LaunchAgents"

        local plist
        plist="$(plist_path "$id")"
        local template="$INSTALL_DIR/client/public/install/services/com.plenumnet.cube.plist.template"

        if [ -f "$template" ]; then
            sed -e "s|__IDENTITY_ID__|$id|g" \
                -e "s|__DAEMON_EXE__|$daemon_exe|g" \
                -e "s|__DAEMON_PORT__|$daemon_port|g" \
                -e "s|__ENGINE_PORT__|$engine_port|g" \
                -e "s|__CRS_URL__|$crs_url|g" \
                -e "s|__IDENTITY_DIR__|$agent_dir|g" \
                -e "s|__LOG_DIR__|$LOG_DIR|g" \
                -e "s|__INSTALL_DIR__|$INSTALL_DIR|g" \
                "$template" > "$plist"
        else
            cat > "$plist" <<PLISTEOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.plenumnet.cube-${id}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${daemon_exe}</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>CUBE_MODE</key>
        <string>cube</string>
        <key>CUBE_API_PORT</key>
        <string>${daemon_port}</string>
        <key>LLM_PORT</key>
        <string>${engine_port}</string>
        <key>CUBE_CRS_URL</key>
        <string>${crs_url}</string>
        <key>RELAY_URL</key>
        <string>${crs_url}</string>
        <key>CUBE_IDENTITY_DIR</key>
        <string>${agent_dir}</string>
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
    <string>${LOG_DIR}/cube-${id}-stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/cube-${id}-stderr.log</string>
    <key>WorkingDirectory</key>
    <string>${INSTALL_DIR}</string>
</dict>
</plist>
PLISTEOF
        fi

        launchctl unload "$plist" 2>/dev/null || true
        launchctl load "$plist"
        echo -e "  ${GREEN}Daemon #$id registered and started as launchd service.${NC}"
        echo -e "  ${DIM}Check status: launchctl list com.plenumnet.cube-${id}${NC}"
        echo -e "  ${DIM}View logs:    tail -f $LOG_DIR/cube-${id}-stdout.log${NC}"
    fi
}

cmd_uninstall() {
    local id="$1"
    echo -e "  Uninstalling daemon #$id service..."

    if [ "$PLATFORM" = "linux" ]; then
        local svc
        svc="$(service_name_linux "$id")"
        sudo systemctl stop "$svc" 2>/dev/null || true
        sudo systemctl disable "$svc" 2>/dev/null || true
        sudo rm -f "/etc/plenumnet/cube-${id}.env"
        if ! ls /etc/plenumnet/cube-*.env &>/dev/null; then
            sudo rm -f /etc/systemd/system/plenumnet-cube@.service
        fi
        sudo systemctl daemon-reload
        echo -e "  ${GREEN}Daemon #$id service removed.${NC}"

    elif [ "$PLATFORM" = "macos" ]; then
        local plist
        plist="$(plist_path "$id")"
        if [ -f "$plist" ]; then
            launchctl unload "$plist" 2>/dev/null || true
            rm -f "$plist"
            echo -e "  ${GREEN}Daemon #$id service removed.${NC}"
        else
            echo -e "  ${YELLOW}No service plist found for identity #$id${NC}"
        fi
    fi
}

if [ $# -lt 1 ]; then
    usage
fi

COMMAND="$1"
ID="${2:-}"

case "$COMMAND" in
    status)
        cmd_status "$ID"
        ;;
    start|stop|restart|logs|install|uninstall)
        if [ -z "$ID" ]; then
            echo -e "${RED}Error: identity ID required for '$COMMAND'${NC}"
            echo "Usage: $0 $COMMAND <identity-id>"
            exit 1
        fi
        "cmd_$COMMAND" "$ID"
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        echo -e "${RED}Unknown command: $COMMAND${NC}"
        usage
        ;;
esac
