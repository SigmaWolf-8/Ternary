#!/usr/bin/env bash
# verify-webhook-tsa.sh — Offline TSA token verification
# No network call required after initial cert download.

set -euo pipefail

PAYLOAD_FILE="${1:?Usage: $0 <payload-file> <tsa-token-b64>}"
TSA_TOKEN_B64="${2:?Usage: $0 <payload-file> <tsa-token-b64>}"
TSA_CERT="${TSA_CERT_FILE:-plenumnet-tsa.pem}"

# Download cert if not cached
if [ ! -f "$TSA_CERT" ]; then
  echo "Downloading PlenumNET TSA certificate..."
  curl -s https://plenumnet.replit.app/api/tsa/certificate/download -o "$TSA_CERT"
fi

# Decode token
echo "$TSA_TOKEN_B64" | base64 -d > /tmp/webhook.tsr

# Compute SHA-256 hash of payload
DIGEST=$(sha256sum "$PAYLOAD_FILE" | awk '{print $1}')

# Verify
echo "Verifying TSA token..."
echo "  Payload hash: $DIGEST"
echo "  Token file:   /tmp/webhook.tsr"
echo "  TSA cert:     $TSA_CERT"
echo ""

openssl ts -verify \
  -digest "$DIGEST" \
  -in /tmp/webhook.tsr \
  -CAfile "$TSA_CERT"

# Exit code 0 = Verification: OK
# Exit code 1 = Verification: FAILED
