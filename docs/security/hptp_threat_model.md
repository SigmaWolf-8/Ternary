<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  Patent(s) Pending — All Rights Reserved
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# HPTP Threat Model & Fallback Chain Analysis

**Version**: 1.0
**Date**: March 15, 2026
**Classification**: Internal / Audit-Ready
**Author**: Timing Engineering, Capomastro Holdings Ltd.
**Status**: Published

---

## 1. Executive Summary

The High-Precision Timing Protocol (HPTP) is a critical subsystem of the Ternary Kernel that provides femtosecond-precision timing for phase encryption, protocol synchronization, and cryptographic operations. This document presents the dedicated threat model for HPTP, including the 5-tier fallback chain architecture, degradation analysis, and anomaly detection thresholds.

HPTP timing integrity is a security-critical property: if timing guarantees are violated, phase encryption windows become exploitable, and cryptographic constant-time guarantees may be undermined.

### HPTP Security Posture

| Metric | Value |
|--------|-------|
| Timing Precision | Femtosecond (10^-15 s) |
| Fallback Tiers | 5 (PTP, NTP, Crystal, Quartz, Cesium) |
| Auto-Escalation Thresholds | 3 levels (Critical >= 8.0, High >= 6.0, Warning >= 4.0) |
| Anomaly Types Tracked | 4 (jitter_variance, clock_drift, sync_failure, glitch_detected) |
| Recovery Time Objective | < 100ms for tier transition |

---

## 2. HPTP Architecture Overview

### 2.1 Timing Source Hierarchy

The HPTP system maintains five independent timing sources organized in a priority-based fallback chain. Each tier provides progressively lower precision but higher availability:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     HPTP Timing Source Hierarchy                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 1: PTP (Precision Time Protocol)                              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Precision: Sub-microsecond (< 1 us)                        │    │
│  │  Latency: 0.5 ms typical                                    │    │
│  │  Jitter Variance: < 5.0 (threshold)                         │    │
│  │  Requirements: Network connectivity, PTP grandmaster         │    │
│  │  Status: PRIMARY - Always preferred when available           │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                              │ DEGRADED or FAILED                    │
│                              ▼                                       │
│  TIER 2: NTP (Network Time Protocol)                                │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Precision: Millisecond range (1-10 ms)                     │    │
│  │  Latency: 12.3 ms typical                                   │    │
│  │  Jitter Variance: < 15.0 (relaxed threshold)                │    │
│  │  Requirements: Network connectivity, NTP server pool         │    │
│  │  Status: FIRST FALLBACK - Higher jitter tolerance            │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                              │ DEGRADED or FAILED                    │
│                              ▼                                       │
│  TIER 3: Crystal Oscillator                                         │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Precision: +/- 2 ppm                                       │    │
│  │  Frequency Accuracy: 2.1 ppm typical                        │    │
│  │  Temperature Range: -10C to +60C                             │    │
│  │  Aging Rate: 0.5 ppm/year                                   │    │
│  │  Requirements: None (local hardware)                         │    │
│  │  Status: OFFLINE FALLBACK - No network required              │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                              │ DEGRADED or FAILED                    │
│                              ▼                                       │
│  TIER 4: Quartz Oscillator                                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Precision: +/- 15 ppm                                      │    │
│  │  Frequency Accuracy: 15.3 ppm typical                       │    │
│  │  Temperature Range: -20C to +70C                             │    │
│  │  Aging Rate: 5.0 ppm/year                                   │    │
│  │  Requirements: None (local hardware, lower power)            │    │
│  │  Status: LOW-POWER FALLBACK                                  │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                              │ DEGRADED or FAILED                    │
│                              ▼                                       │
│  TIER 5: Cesium Reference (Emergency)                               │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Precision: +/- 1e-10 (absolute)                            │    │
│  │  Frequency Accuracy: 1e-10                                  │    │
│  │  Power Consumption: 25W                                      │    │
│  │  Requirements: Specialized hardware, high power              │    │
│  │  Status: EMERGENCY ONLY - Highest accuracy, highest cost     │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Tier Transition Rules

| From | To | Trigger Condition | Recovery Condition |
|------|----|------------------|--------------------|
| PTP | NTP | PTP jitter > 5.0 or sync failure | PTP jitter < 3.0 for 60s |
| NTP | Crystal | NTP unreachable for 30s | NTP reachable and jitter < 10.0 |
| Crystal | Quartz | Crystal frequency drift > 5 ppm | Crystal calibrated and drift < 3 ppm |
| Quartz | Cesium | Quartz frequency drift > 20 ppm | Quartz replaced or recalibrated |
| Any | PTP | PTP available and jitter < 3.0 for 60s | N/A (always preferred) |

### 2.3 Transition State Machine

```
          ┌────────────────────────────────────┐
          │                                    │
          ▼                                    │
    ┌──────────┐    degraded     ┌──────────┐  │  recovery
    │   PTP    │────────────────▶│   NTP    │──┘
    │ (active) │                 │ (active) │
    └──────────┘                 └────┬─────┘
          ▲                           │ failed
          │ recovery                  ▼
          │                    ┌──────────┐
          └────────────────────│ Crystal  │
                               │ (active) │
                               └────┬─────┘
                                    │ failed
                                    ▼
                               ┌──────────┐
                               │  Quartz  │
                               │ (active) │
                               └────┬─────┘
                                    │ failed
                                    ▼
                               ┌──────────┐
                               │  Cesium  │
                               │(emergency)│
                               └──────────┘
```

---

## 3. Threat Vectors

### 3.1 HPTP-Specific Threats

#### T-HPTP-001: GPS/PTP Spoofing

| Field | Value |
|-------|-------|
| **Attack Vector** | RF injection of false PTP/GPS timing signals |
| **Likelihood** | Medium |
| **Impact** | High |
| **Risk Score** | 4.9 |
| **Detection** | Anomaly detection (jitter variance spike) |

**Description**: Attacker injects false timing signals to shift the system clock, undermining phase encryption windows and enabling replay attacks.

**Mitigations**:
- Cross-validation between PTP and NTP sources
- Jitter variance threshold detection (> 5.0 triggers anomaly)
- Authenticated PTP (IEEE 1588 Annex K)
- Automatic fallback to crystal tier if spoofing detected

#### T-HPTP-002: NTP Amplification/Reflection

| Field | Value |
|-------|-------|
| **Attack Vector** | NTP monlist amplification for DoS |
| **Likelihood** | High |
| **Impact** | Medium |
| **Risk Score** | 4.9 |
| **Detection** | Sync failure anomaly type |

**Description**: Attacker exploits NTP monlist for amplification attack, overwhelming the NTP tier and forcing fallback to local oscillators.

**Mitigations**:
- NTP monlist disabled (ntpd configured with `disable monitor`)
- Rate limiting on NTP queries
- Automatic fallback to crystal tier preserves timing
- Anomaly detection flags sync failures

#### T-HPTP-003: Oscillator Aging/Drift

| Field | Value |
|-------|-------|
| **Attack Vector** | Natural crystal/quartz aging beyond spec |
| **Likelihood** | Low |
| **Impact** | Medium |
| **Risk Score** | 1.5 |
| **Detection** | Clock drift anomaly type |

**Description**: Over time, crystal and quartz oscillators age, increasing frequency drift beyond acceptable tolerances. This is a reliability concern rather than an attack vector.

**Mitigations**:
- Continuous drift monitoring via HPTP anomaly service
- Temperature compensation algorithms
- Scheduled replacement at manufacturer intervals
- Automatic escalation when drift exceeds threshold

#### T-HPTP-004: Glitch Injection

| Field | Value |
|-------|-------|
| **Attack Vector** | Voltage/clock glitching via physical access |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Detection** | Glitch detected anomaly type |

**Description**: Attacker with physical access induces voltage or clock glitches to cause timing-dependent security checks to fail, potentially bypassing phase encryption windows.

**Mitigations**:
- Glitch detection circuits in HPTP secure element
- Redundant timing source cross-validation
- Automatic shutdown on detected glitch patterns
- Escalation to audit log (severity >= 8.0)

#### T-HPTP-005: Secure Element Compromise

| Field | Value |
|-------|-------|
| **Attack Vector** | Hardware timing source manipulation |
| **Likelihood** | Low |
| **Impact** | Critical |
| **Risk Score** | 3.2 |
| **Detection** | Multiple anomaly types correlated |

**Description**: Complete compromise of the HPTP secure element through supply chain attack or advanced persistent threat. Cross-references THREAT_010 in the main threat model.

**Mitigations**:
- 5-tier fallback chain provides resilience
- Anomaly detection with auto-escalation
- External audit by Riscure (planned Q2 2026)
- Hardware attestation framework (planned)

---

## 4. Auto-Escalation Logic

### 4.1 Severity Score Thresholds

The HPTP anomaly detection system uses a 0-10 severity score to determine escalation:

```
Severity Score    │  Classification  │  Action
──────────────────┼──────────────────┼──────────────────────────────
 >= 8.0           │  CRITICAL        │  Immediate audit log entry
                  │                  │  severity: "critical"
                  │                  │  escalation_triggered: true
                  │                  │  Notify security team immediately
──────────────────┼──────────────────┼──────────────────────────────
 >= 6.0 (and      │  HIGH            │  Audit log entry
  duration > 5m)  │                  │  severity: "high"
                  │                  │  escalation_triggered: true
                  │                  │  Notify within 1 hour
──────────────────┼──────────────────┼──────────────────────────────
 >= 4.0           │  WARNING         │  Audit log entry
                  │                  │  severity: "warning"
                  │                  │  escalation_triggered: false
                  │                  │  Log only
──────────────────┼──────────────────┼──────────────────────────────
 < 4.0            │  INFO            │  No audit log entry
                  │                  │  escalation_triggered: false
                  │                  │  Informational only
──────────────────┴──────────────────┴──────────────────────────────
```

### 4.2 Cross-Service Escalation Flow

```
HPTP Anomaly Detected
        │
        ▼
┌───────────────┐    severity >= 8.0    ┌─────────────────────┐
│  Severity     │──────────────────────▶│  Security Audit Log │
│  Assessment   │                       │  severity: critical │
│               │    severity >= 6.0    │  category: hptp     │
│               │───(duration > 5m)────▶│  severity: high     │
│               │                       └─────────┬───────────┘
│               │    severity >= 4.0              │
│               │──────────────────────▶ warning   │
│               │                       (log only) │
│               │    severity < 4.0               │
│               │──────────────────────▶ info      ▼
└───────────────┘                    ┌──────────────────┐
                                     │  Dashboard Alert  │
                                     │  (real-time)      │
                                     └──────────────────┘
```

---

## 5. Anomaly Type Analysis

### 5.1 Jitter Variance

| Parameter | Value |
|-----------|-------|
| **Detection Threshold** | Variance > 5.0 (PTP), > 15.0 (NTP) |
| **Severity Mapping** | Linear: variance 5.0 = score 4.0, variance 10.0 = score 8.0 |
| **Impact** | Degrades phase encryption timing windows |
| **Recovery** | Re-synchronize with grandmaster; fallback if persistent |

### 5.2 Clock Drift

| Parameter | Value |
|-----------|-------|
| **Detection Threshold** | > 5 ppm (crystal), > 20 ppm (quartz) |
| **Severity Mapping** | Based on tier: crystal drift 5 ppm = score 5.0 |
| **Impact** | Cumulative timing error affects protocol synchronization |
| **Recovery** | Temperature compensation; oscillator replacement if aging |

### 5.3 Sync Failure

| Parameter | Value |
|-----------|-------|
| **Detection Threshold** | 3 consecutive sync attempts failed |
| **Severity Mapping** | Immediate score 6.0 (network tiers), 4.0 (local tiers) |
| **Impact** | Loss of external time reference |
| **Recovery** | Automatic fallback to next tier; retry after 60s |

### 5.4 Glitch Detected

| Parameter | Value |
|-----------|-------|
| **Detection Threshold** | Voltage/clock anomaly in secure element |
| **Severity Mapping** | Immediate score 9.0 (potential attack) |
| **Impact** | Security boundary violation possible |
| **Recovery** | System halt for investigation; manual restart required |

---

## 6. Fallback Chain Performance Metrics

### 6.1 Per-Tier Availability Targets

| Tier | Availability Target | Latency Budget | Accuracy |
|------|-------------------|----------------|----------|
| PTP | 99.9% | < 1 ms | Sub-microsecond |
| NTP | 99.5% | < 50 ms | 1-10 ms |
| Crystal | 99.99% | N/A (local) | +/- 2 ppm |
| Quartz | 99.99% | N/A (local) | +/- 15 ppm |
| Cesium | 99.999% | N/A (local) | +/- 1e-10 |

### 6.2 Fallback Analysis API

The HPTP service provides programmatic access to fallback chain performance:

```
GET /api/security/hptp/fallback-analysis
Response: {
  ptp: { availability: 99.8%, avg_latency_ms: 0.5, jitter_variance: 2.1 },
  ntp: { availability: 95.2%, avg_latency_ms: 12.3, jitter_variance: 8.5 },
  crystal: { avg_frequency_ppm: 2.1, temperature_c: 24.5 },
  quartz: { avg_frequency_ppm: 15.3, temperature_c: 25.1 },
  cesium: { frequency_accuracy: "1e-10", power_consumption_w: 25.0 }
}
```

---

## 7. Redundancy Architecture

### 7.1 N+4 Redundancy Model

The HPTP system provides N+4 redundancy (5 tiers for 1 required timing source):

```
┌─────────────────────────────────────────┐
│         Redundancy Coverage             │
│                                         │
│  PTP ═══════════════════  (Primary)     │
│  NTP ═══════════════════  (Hot Standby) │
│  Crystal ═══════════════  (Warm Standby)│
│  Quartz ════════════════  (Cold Standby)│
│  Cesium ════════════════  (Emergency)   │
│                                         │
│  Required: 1 active source              │
│  Available: 5 sources                   │
│  Redundancy: N+4                        │
│  MTBF: > 100,000 hours (combined)       │
└─────────────────────────────────────────┘
```

### 7.2 Failure Mode Analysis

| Failure Scenario | Sources Lost | Remaining | Impact |
|-----------------|-------------|-----------|--------|
| Network outage | PTP, NTP | Crystal, Quartz, Cesium | Degraded precision, operational |
| Single hardware failure | 1 local source | 4 remaining | Minimal impact |
| Dual failure (network + crystal) | PTP, NTP, Crystal | Quartz, Cesium | Reduced precision, operational |
| Catastrophic (4 tiers) | 4 sources | Cesium | Emergency mode, full accuracy |
| Total loss (all 5) | All | None | System halt (safety shutdown) |

---

## 8. Monitoring Integration

### 8.1 Dashboard Endpoints

```
GET /api/security/hptp/status           - Current health and active tier
GET /api/security/hptp/anomalies        - Anomaly history (7-day default)
GET /api/security/hptp/fallback-analysis - Per-tier performance metrics
GET /api/security/hptp/stats            - Aggregate statistics
GET /api/security/hptp/thresholds       - Current threshold configuration
GET /api/security/hptp/redundancy       - Redundancy architecture info
```

### 8.2 Alert Conditions

| Condition | Threshold | Action |
|-----------|-----------|--------|
| Critical anomaly | severity >= 8.0 | Immediate page to security team |
| Tier degradation | Active tier != PTP for > 5 min | High-priority notification |
| Escalation rate | > 3 escalations in 1 hour | Pattern analysis triggered |
| Resolution SLA | Unresolved > 24 hours | Auto-escalate to management |

---

## 9. Research References

1. IEEE 1588-2019: Precision Time Protocol (PTPv2.1)
2. NIST SP 800-82: Guide to ICS Security (timing requirements)
3. Mills, D. et al.: "Network Time Protocol Version 4" (RFC 5905)
4. CCCS ITSP.40.111: HPTP Security Considerations
5. Osmocom Project: Timing reference for mobile infrastructure

---

*Document Control: This threat model is maintained by Timing Engineering and reviewed quarterly. Updates require sign-off from the Security Lead and Architecture Review Board.*
