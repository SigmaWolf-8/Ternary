# Module Guide: High-Precision Timing Protocol (HPTP)

**Module:** `salvi_timing`  
**Status:** Complete (P3-006 to P3-009)  
**Tests:** ~55 tests

---

## Overview

The High-Precision Timing Protocol (HPTP) provides femtosecond-accurate time synchronization for the ternary network. It integrates with optical lattice clocks and includes regulatory certification tools for financial compliance.

### Key Features

- **Femtosecond Synchronization** — Sub-picosecond offset calculation
- **Optical Clock Manager** — Support for Sr, Yb, Al, Hg lattice clocks
- **Regulatory Certification** — FINRA 613 and MiFID II compliance
- **Phase-Coherent Distribution** — Ternary-aware time propagation

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│          (Timestamps, event ordering, auditing)             │
├─────────────────────────────────────────────────────────────┤
│                    HPTP Protocol Layer                       │
│   ┌────────────────────────────────────────────────────┐   │
│   │ Sync Protocol │ Offset Calc │ Phase Alignment      │   │
│   └────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                Optical Clock Manager                         │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│   │Strontium│  │Ytterbium│  │Aluminum │  │ Mercury │       │
│   │  87Sr   │  │  171Yb  │  │  27Al+  │  │  199Hg  │       │
│   └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
├─────────────────────────────────────────────────────────────┤
│              Certification & Compliance                      │
│   ┌────────────────────────────────────────────────────┐   │
│   │ FINRA 613 (50ms) │ MiFID II (100us/1ms)            │   │
│   └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## HPTP Synchronization

### Time Model

```
┌───────────────────────────────────────────────────┐
│  Stratum 0: Optical Lattice Clocks (~10^-18)      │
├───────────────────────────────────────────────────┤
│  Stratum 1: Primary Time Servers (~10^-15)        │
├───────────────────────────────────────────────────┤
│  Stratum 2: Secondary Time Servers (~10^-12)      │
├───────────────────────────────────────────────────┤
│  Stratum 3+: End Clients (~10^-9)                 │
└───────────────────────────────────────────────────┘
```

### Sync Protocol

```rust
use salvi_timing::hptp::{HptpClient, HptpConfig, SyncResult};

let config = HptpConfig {
    primary_server: "hptp://time1.tern:3319".parse()?,
    backup_servers: vec![
        "hptp://time2.tern:3319".parse()?,
        "hptp://time3.tern:3319".parse()?,
    ],
    sync_interval: Duration::from_secs(1),
    max_offset: Duration::from_micros(100),
};

let client = HptpClient::new(config)?;
client.start()?;

let now = client.now();
let status = client.status();
println!("Offset: {:?}", status.offset);
println!("Stratum: {}", status.stratum);
println!("Accuracy: {:?}", status.estimated_accuracy);
```

### Offset Calculation

```rust
use salvi_timing::hptp::offset::{OffsetCalculator, TimestampPair};

impl OffsetCalculator {
    pub fn calculate(&self, exchange: &TimestampExchange) -> Offset {
        // T1: Client send, T2: Server receive, T3: Server send, T4: Client receive
        let delay = (exchange.t4 - exchange.t1) - (exchange.t3 - exchange.t2);
        let offset = ((exchange.t2 - exchange.t1) + (exchange.t3 - exchange.t4)) / 2;
        let torsion_correction = self.calculate_torsion_correction(exchange);
        
        Offset {
            value: offset + torsion_correction,
            uncertainty: delay / 2,
            confidence: self.calculate_confidence(exchange),
        }
    }
    
    fn calculate_torsion_correction(&self, exchange: &TimestampExchange) -> Duration {
        let forward_torsion = exchange.forward_route_info().total_torsion_weight();
        let reverse_torsion = exchange.reverse_route_info().total_torsion_weight();
        let asymmetry = forward_torsion - reverse_torsion;
        Duration::from_femtos((asymmetry * TORSION_TIME_FACTOR) as i128)
    }
}
```

---

## Optical Clock Manager

### Supported Clock Types

| Clock | Element | Transition | Accuracy |
|-------|---------|------------|----------|
| Strontium | 87Sr | 698 nm | 10^-18 |
| Ytterbium | 171Yb | 578 nm | 10^-18 |
| Aluminum | 27Al+ | 267 nm | 10^-19 |
| Mercury | 199Hg | 266 nm | 10^-18 |

### Clock Configuration

```rust
use salvi_timing::optical::{OpticalClockManager, ClockType, ClockConfig};

let manager = OpticalClockManager::new()?;

let sr_config = ClockConfig {
    clock_type: ClockType::Strontium87,
    transition_frequency: 429_228_004_229_873.0,
    systematic_uncertainty: 2.0e-18,
    statistical_uncertainty: 1.0e-17,
    calibration_interval: Duration::from_hours(24),
};

manager.add_clock("sr_primary", sr_config)?;

let yb_config = ClockConfig {
    clock_type: ClockType::Ytterbium171,
    transition_frequency: 518_295_836_590_863.0,
    systematic_uncertainty: 3.0e-18,
    statistical_uncertainty: 2.0e-17,
    calibration_interval: Duration::from_hours(12),
};

manager.add_clock("yb_backup", yb_config)?;
manager.start_comparison()?;
```

### Clock Ensemble

```rust
use salvi_timing::optical::ensemble::{ClockEnsemble, EnsembleConfig};

let ensemble = ClockEnsemble::new(EnsembleConfig {
    clocks: vec!["sr_primary", "yb_backup", "al_reference"],
    weighting: WeightingScheme::ByUncertainty,
    outlier_rejection: OutlierRejection::MadBased(3.0),
})?;

let time = ensemble.get_time()?;
println!("Ensemble time: {:?}", time);
```

---

## Regulatory Compliance

### FINRA Rule 613 (CAT NMS)

Requires synchronization within 50 milliseconds of NIST:

```rust
use salvi_timing::compliance::{FinraCompliance, ComplianceReport};

let finra = FinraCompliance::new(client.clone())?;

let report = finra.generate_report()?;
println!("Compliant: {}", report.is_compliant);
println!("Max offset: {:?}", report.max_observed_offset);
println!("Required: < 50ms");

finra.enable_continuous_monitoring()?;
```

### MiFID II (EU Markets)

Requires synchronization based on trading activity:

| Activity | Requirement |
|----------|-------------|
| High-frequency trading | Within 100 microseconds of UTC |
| Non-HFT trading | Within 1 millisecond of UTC |
| Trade reporting | Within 1 second of UTC |

```rust
use salvi_timing::compliance::{MifidCompliance, TradingActivity};

let mifid = MifidCompliance::new(client.clone())?;

let report = mifid.generate_report(TradingActivity::HighFrequency)?;
println!("MiFID II compliant: {}", report.is_compliant);
println!("Max offset: {:?}", report.max_observed_offset);
println!("Required: < 100us for HFT");
```

### Certification

```rust
use salvi_timing::compliance::Certification;

let cert = Certification::new()?;

let certificate = cert.generate(
    CertificationType::FinraRule613,
    &audit_log,
    &compliance_report,
)?;

certificate.sign(&signing_key)?;
certificate.export("compliance_cert.pem")?;
```

---

## Timestamps

### Femtosecond Timestamps

```rust
use salvi_timing::FemtosecondTimestamp;

let now = FemtosecondTimestamp::now();
println!("Time: {} fs since epoch", now.femtoseconds());
println!("Human: {}", now.to_rfc3339());

let later = now.add(Duration::from_micros(100));
let diff = later.duration_since(now);
println!("Difference: {:?}", diff);
```

### Timestamp Ordering

```rust
use salvi_timing::ordering::{CausalOrder, HappensBeforeRelation};

let order = CausalOrder::new();

let ts1 = order.stamp_event("transaction_created")?;
let ts2 = order.stamp_event("transaction_confirmed")?;

assert!(ts1.happens_before(&ts2));
```

---

## Best Practices

### 1. Use Appropriate Stratum
Don't connect all clients directly to Stratum 0 clocks.

### 2. Monitor Compliance Continuously
Enable real-time compliance monitoring for regulated environments.

### 3. Use Clock Ensembles
Multiple clocks provide better accuracy through averaging.

### 4. Account for Torsion Asymmetry
The torsion correction is essential for accurate synchronization in the ternary network.

---

*Così sia.*
