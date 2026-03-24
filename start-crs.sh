#!/bin/bash
# PlenumNET CRS Daemon Starter
# Uses systemd when available, falls back to nohup

CRS_SERVICE="plenumnet-crs.service"
CRS_PORT="${CUBE_API_PORT:-8181}"
DAEMON_BIN="/home/runner/workspace/target/release/inter-cube-daemon"

if command -v systemctl &>/dev/null && [ -f "/etc/systemd/system/$CRS_SERVICE" ]; then
    echo "Starting CRS daemon via systemd..."
    sudo systemctl restart "$CRS_SERVICE"
    sleep 2
    systemctl status "$CRS_SERVICE" --no-pager
    echo ""
    echo "View logs: journalctl -u $CRS_SERVICE -f"
else
    pkill -f "inter-cube-daemon" 2>/dev/null || true
    sleep 1
    CUBE_MODE=crs CUBE_API_PORT="$CRS_PORT" CUBE_IDENTITY_PASSPHRASE="plenumlan-prototype-2026" \
      nohup "$DAEMON_BIN" > /tmp/crs-daemon.log 2>&1 &
    echo "CRS daemon started (PID $!, port $CRS_PORT, mode=crs)"
    sleep 2
    curl -s "http://127.0.0.1:${CRS_PORT}/health"
    echo ""
    echo "NOTE: Running as background process (nohup). To run as a service,"
    echo "install the systemd unit file and use systemctl."
fi
