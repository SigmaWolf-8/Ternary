# Payment & Witnessing Architecture

## Overview

The Salvi Framework Payment & Witnessing Architecture provides a complete solution for payment processing with blockchain-backed audit trails and regulatory compliance.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PAYMENT GATEWAYS                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
│  │  Stripe  │  │  Interac │  │  Crypto  │                                   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                                   │
│       │             │             │                                         │
│       └─────────────┼─────────────┘                                         │
│                     ▼                                                       │
│          ┌──────────────────┐                                               │
│          │ Payment Listener │ (Port 3001)                                   │
│          │  HMAC Validation │                                               │
│          └────────┬─────────┘                                               │
│                   │                                                         │
│                   ▼                                                         │
│          ┌──────────────────┐    ┌────────────────────┐                     │
│          │  SFK Core API    │───▶│ Femtosecond Service │ (Port 3006)        │
│          │  (Port 3002)     │    │   HPTP Protocol     │                    │
│          │  State Machine   │    └──────────┬─────────┘                     │
│          └────────┬─────────┘               │                               │
│                   │                         ▼                               │
│                   │              ┌─────────────────────┐                    │
│                   │              │ Certification Svc   │ (Port 3007)        │
│                   │              │ FINRA 613 / MiFID II│                    │
│                   │              └─────────────────────┘                    │
│                   │                                                         │
│    ┌──────────────┼──────────────┬─────────────────────┐                    │
│    ▼              ▼              ▼                     ▼                    │
│ ┌────────┐  ┌─────────┐  ┌──────────────┐  ┌────────────────┐               │
│ │ Hedera │  │  XRPL   │  │  Algorand    │  │ Oracle Bridge  │               │
│ │  HCS   │  │Settlement│  │ Contracts    │  │ Hedera→Algo    │               │
│ │ (3003) │  │ (3004)  │  │   (3005)     │  │                │               │
│ └────────┘  └─────────┘  └──────────────┘  └────────────────┘               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Microservices

### 1. Payment Listener (Port 3001)

Handles webhooks from payment gateways with secure signature verification.

**Security Features:**
- HMAC-SHA256 for Stripe and Crypto webhooks
- HMAC-SHA512 for Interac e-Transfer
- Constant-time signature comparison
- Idempotency key enforcement

**Endpoints:**
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/webhook/stripe` | POST | Stripe payment webhooks |
| `/webhook/interac` | POST | Interac e-Transfer webhooks |
| `/webhook/crypto` | POST | Cryptocurrency webhooks |
| `/api/payments/:id/status` | GET | Payment status lookup |
| `/health` | GET | Service health check |

### 2. SFK Core API (Port 3002)

Orchestrates payment operations through the blockchain witnessing pipeline.

**Features:**
- OpenAPI/Swagger documentation
- State machine for operation lifecycle
- Femtosecond-precision timestamps
- Unified metadata across chains

**Operation Types:**
- `payment_witness` - Witness payment on Hedera
- `data_attest` - Attest data hash
- `state_transition` - Record state change
- `consensus_round` - Consensus participation

**Security Modes:**
- `mode_zero` - No additional security (testing)
- `mode_one` - Standard witnessing
- `phi_plus` - Maximum security (multi-chain)

### 3. Hedera HCS Service (Port 3003)

Immutable witnessing on Hedera Hashgraph Consensus Service.

**Features:**
- Topic-based consensus
- Running hash verification
- Femtosecond timestamp attachment
- Mirror node integration

### 4. XRPL Service (Port 3004)

Payment settlement on XRP Ledger.

**Features:**
- Cross-border settlement
- XRP and issued currency support
- Real-time transaction validation
- Escrow and payment channels

### 5. Algorand Service (Port 3005)

Smart contract execution on Algorand.

**Smart Contracts:**
- **TAT-GOV-001**: Ternary governance voting (A/B/C = For/Against/Abstain)
- **TAT-DIST-001**: Efficiency-based reward distribution (58.5% bonus)

### 6. Femtosecond Service (Port 3006)

High-precision timing with HPTP synchronization.

**Features:**
- Sub-microsecond network synchronization
- Multi-peer consensus timing
- Drift compensation (linear regression)
- Jitter filtering

**Clock Sources (Production):**
- GPS disciplined oscillators (GPSDO)
- PTP hardware timestamping NICs
- Atomic reference clocks (Cs/Rb)

### 7. Certification Service (Port 3007)

Regulatory compliance certification.

**Compliance Standards:**
- FINRA Rule 613 (CAT): ≤50ms drift from NIST
- MiFID II RTS 25 (HFT): ≤1ms divergence
- MiFID II (general): ≤100μs

## Deployment

### Docker Compose (Development/Staging)

```bash
cd deployments/docker
cp .env.example .env
# Edit .env with your credentials
docker-compose -f docker-compose.production.yml up -d
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret | Yes |
| `INTERAC_WEBHOOK_SECRET` | Interac webhook secret | Yes |
| `HEDERA_OPERATOR_ID` | Hedera account ID | Yes |
| `HEDERA_OPERATOR_KEY` | Hedera private key | Yes |
| `XRPL_WALLET_SEED` | XRP wallet seed | Yes |
| `ALGORAND_MNEMONIC` | Algorand wallet mnemonic | Yes |
| `SIGNING_SECRET_KEY` | Certificate signing key | Yes (prod) |

## Kong API Gateway

All services are exposed through Kong with:
- Rate limiting per endpoint
- IP restriction for internal services
- JWT authentication for mutations
- Request/response logging

See `kong/services/` for route configurations.

## Compliance

### FINRA Rule 613 Requirements

1. Clock synchronization to NIST atomic clock within 50ms
2. Timestamps recorded at reportable event time
3. Millisecond granularity minimum
4. Audit trail retention

### MiFID II RTS 25 Requirements

1. Synchronization to UTC via designated time sources
2. Maximum divergence based on activity type
3. Microsecond timestamp granularity
4. Traceability to UTC source

## Testing

```bash
cd tests/integration
npm install
npm test
```

Individual test suites:
```bash
npm run test:payment    # Payment flow tests
npm run test:timing     # Timing certification tests
```
