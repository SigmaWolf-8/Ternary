# PlenumNET Kong Konnect Configuration

This directory contains Kong Gateway configuration for the PlenumNET Framework APIs.

## Overview

Kong Konnect provides API gateway capabilities including:
- **Rate Limiting** - Protect APIs from abuse
- **Authentication** - API keys for AI agents and consumers
- **Routing** - Intelligent request routing
- **Analytics** - Monitor API usage and performance

## Files

| File | Description |
|------|-------------|
| `kong.yaml` | Main Kong declarative configuration (decK format) |
| `README.md` | This documentation file |

## Services Configured

| Service | Endpoint | Description |
|---------|----------|-------------|
| `plenumnet-timing` | `/api/timing` | Femtosecond-precision timing APIs (FINRA CAT compliant) |
| `plenumnet-ternary` | `/api/ternary` | Ternary computing operations (GF(3) arithmetic) |
| `plenumnet-phase` | `/api/phase` | Phase-split encryption for quantum-safe data |
| `plenumnet-demo` | `/api/demo` | Compression demonstration APIs |

## Rate Limits

| Service | Requests/Minute | Requests/Hour |
|---------|-----------------|---------------|
| Timing API | 100 | 1,000 |
| Ternary API | 200 | 2,000 |
| Phase API | 100 | 1,000 |
| Demo API | 50 | 500 |

## For AI Agents

### Accessing PlenumNET APIs through Kong

1. **Get your API key** from the PlenumNET admin or use the default AI agent key
2. **Include the key in requests**:
   ```bash
   curl -X GET "https://your-kong-gateway/api/timing/timestamp" \
     -H "apikey: your-api-key"
   ```

### Available Endpoints

```bash
# Get femtosecond timestamp
GET /api/timing/timestamp

# Get timing metrics
GET /api/timing/metrics

# Ternary operations
POST /api/ternary/add
POST /api/ternary/multiply
POST /api/ternary/convert

# Phase encryption
POST /api/phase/split
POST /api/phase/recombine
GET /api/phase/config/:mode
```

### Configuration via GitHub

This `kong.yaml` file can be synced to Kong Konnect using:

```bash
# Install decK CLI
brew install kong/deck/deck

# Sync configuration to Kong Konnect
deck gateway sync kong.yaml \
  --konnect-control-plane-name "your-control-plane" \
  --konnect-token "$KONG_KONNECT_TOKEN"

# Diff before applying
deck gateway diff kong.yaml \
  --konnect-control-plane-name "your-control-plane" \
  --konnect-token "$KONG_KONNECT_TOKEN"
```

### PlenumNET Admin Interface

You can also sync this configuration through the PlenumNET web interface:
1. Go to `/kong-konnect`
2. Click "Sync Configuration"
3. Select your control plane
4. The configuration will be applied automatically

## Security Notes

- API keys should be stored securely and rotated regularly
- Rate limits can be adjusted based on your tier
- All PlenumNET APIs use quantum-safe protocols

## Copyright

© Capomastro Holdings Ltd 2026. All Rights Reserved.
