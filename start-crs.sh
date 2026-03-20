#!/bin/bash
pkill -f "inter-cube-daemon" 2>/dev/null || true
sleep 1
CUBE_MODE=crs CUBE_API_PORT=8181 CUBE_IDENTITY_PASSPHRASE="plenumlan-prototype-2026" \
  nohup /home/runner/workspace/target/release/inter-cube-daemon > /tmp/crs-daemon.log 2>&1 &
echo "CRS daemon started (PID $!, port 8181, mode=crs)"
sleep 2
curl -s http://127.0.0.1:8181/health
echo ""
