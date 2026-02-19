# PlenumNET Tonal Diffusion API Reference

**Version**: 1.0  
**Base Path**: `/api`  
**Authentication**: None (public endpoints)  
**Content-Type**: `application/json`

---

## Tonal Field Endpoints

### GET /api/tonal/field

Returns the current tonal field state at the local node, including the computed potential, gradient vector, neighbor count, and last update timestamp.

**Response** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `potential` | `number` | Aggregate tonal potential (weighted average of neighbor contributions) |
| `gradient` | `object` | Three-component gradient vector `{ eta, theta, psi }` |
| `gradient.eta` | `number` | Gradient along radial axis (distance from core) |
| `gradient.theta` | `number` | Gradient along poloidal axis (regional group) |
| `gradient.psi` | `number` | Gradient along toroidal axis (ring position) |
| `neighborCount` | `number` | Number of active neighbor nodes contributing to the field |
| `lastUpdate` | `number` | Unix timestamp (ms) of the most recent neighbor update |

**Example Response**

```json
{
  "potential": 0.0342,
  "gradient": { "eta": -0.012, "theta": 0.003, "psi": -0.001 },
  "neighborCount": 6,
  "lastUpdate": 1740000000000
}
```

---

### GET /api/tonal/neighbors

Returns all neighbor states as a keyed object. Each entry represents a node contributing to the local tonal field.

**Response** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `<nodeId>` | `object` | Neighbor state keyed by node identifier |
| `.potential` | `number` | Computed potential contribution from this neighbor |
| `.coherence` | `number` | Sync coherence quality (0.0 - 1.0) |
| `.distance` | `object` | Toroidal address `{ eta, theta, psi }` |
| `.lastUpdate` | `number` | Unix timestamp (ms) of last packet from this neighbor |

**Example Response**

```json
{
  "node-alpha": {
    "potential": 0.045,
    "coherence": 0.92,
    "distance": { "eta": 1.2, "theta": 0.5, "psi": 2.1 },
    "lastUpdate": 1740000000000
  }
}
```

---

### POST /api/tonal/packet

Submit an FM timing packet from a network node. Updates the local tonal field with the sender's timing state.

**Request Body**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `nodeId` | `string` | Yes | Unique identifier of the sending node |
| `packet` | `object` | Yes | FM timing packet payload |
| `packet.frequencyState` | `object` | Yes | Current oscillator state |
| `packet.frequencyState.f_inst` | `number` | Yes | Instantaneous frequency (Hz) |
| `packet.frequencyState.sidebands` | `number[4]` | Yes | Power at f +/- nf_m harmonics |
| `packet.frequencyState.coherence` | `number` | Yes | Sync quality (0.0 - 1.0) |
| `packet.modulationIndex` | `integer` | Yes | FM modulation index beta (0 - 255) |
| `packet.networkHealth` | `-1 \| 0 \| 1` | Yes | Aggregate network health trit |
| `packet.entropyNonce` | `integer[8]` | Yes | 8-byte entropy nonce (values 0-255) |
| `address` | `object` | Yes | Sender's toroidal address `{ eta, theta, psi }` |

**Example Request**

```json
{
  "nodeId": "node-alpha",
  "packet": {
    "frequencyState": {
      "f_inst": 1.0,
      "sidebands": [0.1, 0.05, 0.02, 0.01],
      "coherence": 0.95
    },
    "modulationIndex": 128,
    "networkHealth": 1,
    "entropyNonce": [12, 45, 78, 91, 33, 67, 88, 42]
  },
  "address": { "eta": 0.5, "theta": 1.2, "psi": 0.3 }
}
```

**Response** `200 OK`

```json
{
  "accepted": true,
  "potential": 0.0342
}
```

**Error** `400 Bad Request`

```json
{
  "error": "Invalid packet data",
  "details": [...]
}
```

---

## Resonance Endpoints

### GET /api/resonance/status

Returns the current state of the adaptive resonance detector, including sync rate, detected resonant frequency, and network wave speed.

**Response** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `currentSyncRate` | `number` | Active synchronization rate (Hz) |
| `resonantFrequency` | `number` | Detected quarter-wave resonant frequency (Hz) |
| `networkWaveSpeed` | `number` | Estimated propagation speed (path-units/sec) |
| `medianRttMs` | `number` | Median round-trip time across recent samples (ms) |
| `optimalSyncRate` | `number` | Computed optimal sync rate from Moens-Korteweg analog |
| `rttSamples` | `number` | Number of RTT measurements in the ring buffer |

**Example Response**

```json
{
  "currentSyncRate": 125.0,
  "resonantFrequency": 125.0,
  "networkWaveSpeed": 5000.0,
  "medianRttMs": 10.0,
  "optimalSyncRate": 250.0,
  "rttSamples": 42
}
```

---

### POST /api/resonance/sweep

Performs an FM frequency sweep across +/-20% of the detected resonant frequency. Finds and applies the optimal sync rate.

**Request Body**: None required

**Response** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `optimalRate` | `number` | Best sync frequency found during sweep |
| `qualityAtOptimal` | `number` | Resonance quality factor at optimal rate |
| `sweepRange` | `[number, number]` | Low and high bounds of the sweep range |
| `samples` | `array` | Measured `{ frequency, quality }` pairs |

**Example Response**

```json
{
  "optimalRate": 125.0,
  "qualityAtOptimal": 10.0,
  "sweepRange": [100.0, 150.0],
  "samples": [
    { "frequency": 100.0, "quality": 1.78 },
    { "frequency": 112.5, "quality": 3.45 },
    { "frequency": 125.0, "quality": 10.0 },
    { "frequency": 137.5, "quality": 3.21 },
    { "frequency": 150.0, "quality": 1.62 }
  ]
}
```

---

### POST /api/resonance/rtt

Records a round-trip time measurement. Used to calibrate network wave speed and resonant frequency detection.

**Request Body**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rttMs` | `number` | Yes | Round-trip time in milliseconds (must be positive) |

**Example Request**

```json
{ "rttMs": 12.5 }
```

**Response** `200 OK`

```json
{ "recorded": true, "medianRttMs": 11.25 }
```

**Error** `400 Bad Request`

```json
{ "error": "rttMs must be a positive number" }
```

---

## Metrics Endpoints

### GET /api/metrics/plenum

Returns the four dimensionless Plenum parameters (Pi1-Pi4) with health assessment. These are the real-time KPIs from Section 8.2 of the unified tonal diffusion model.

**Response** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `metrics.pi1` | `number` | Field/Pressure ratio (target ~1e-6) |
| `metrics.pi2` | `number` | Sync/Resonance ratio (target ~1.0) |
| `metrics.pi3` | `number` | Information density (target >>1) |
| `metrics.pi4` | `number` | Tonal authority (context-dependent) |
| `health.status` | `string` | `"healthy"`, `"warning"`, or `"critical"` |
| `health.issues` | `string[]` | List of detected issues |

**Example Response**

```json
{
  "metrics": {
    "pi1": 3.42e-7,
    "pi2": 0.98,
    "pi3": 150.0,
    "pi4": 0.045
  },
  "health": {
    "status": "healthy",
    "issues": []
  }
}
```

---

## Data Types Reference

### TernaryTrit

Three-valued balanced ternary type used throughout the system:

| Value | Meaning | Rust | TypeScript |
|-------|---------|------|------------|
| -1 | Neg / Behind / Falling | `TernaryTrit::Neg` | `-1` |
| 0 | Zero / Synchronized / Flat | `TernaryTrit::Zero` | `0` |
| +1 | Pos / Ahead / Rising | `TernaryTrit::Pos` | `1` |

### ToroidalAddress

Three-component address on the toroidal network mesh:

| Component | Range | Meaning |
|-----------|-------|---------|
| `eta` | 0+ | Radial distance from core timing authority |
| `theta` | 0 - 2pi | Poloidal angle (regional group) |
| `psi` | 0 - 2pi | Toroidal angle (ring position) |
