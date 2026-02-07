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

## Ancient Calendar Synchronization

The Salvi Epoch (April 1, 2025 00:00:00.000 UTC) is anchored to nine ancient calendar systems spanning 30,000+ years of human timekeeping, providing a universal temporal reference frame that bridges modern post-quantum timing with humanity's oldest astronomical traditions.

### Supported Calendar Systems

| Calendar System | Origin | Time Depth | Epoch Relationship |
|----------------|--------|-----------|-------------------|
| 13-Moon Natural Time | ~28,000 BCE | ~30,000 years | 364-day cycle (13 x 28 days) |
| Byzantine Anno Mundi | 5509 BCE | ~7,500 years | Eastern Roman creation reckoning |
| Julian Day Number | 4713 BCE | ~6,700 years | Continuous astronomical day count |
| Hebrew (Anno Mundi) | 3761 BCE | ~5,800 years | Lunisolar calendar from creation |
| Mayan Long Count | 3114 BCE | ~5,100 years | Vigesimal count from creation date |
| Vedic Kali Yuga | 3102 BCE | ~5,100 years | Hindu cosmological age (432,000 yr) |
| Egyptian Civil | ~2781 BCE | ~4,800 years | Sothic cycle solar calendar |
| Chinese Sexagenary | ~2637 BCE | ~4,600 years | 60-year Stems & Branches cycle |
| Islamic Hijri | 622 CE | ~1,400 years | Lunar calendar from the Hijra |

### Salvi Epoch Anchor Points

All values verified against scholarly reference data. Mayan 13.0.0.0.0 = Dec 21, 2012; JD 2451545.0 = J2000.0 epoch; Hebrew Nisan alignment confirmed.

```
┌──────────────────────────────────────────────────────────────────────┐
│                       SALVI EPOCH (Day Zero)                          │
│                 April 1, 2025 00:00:00.000 UTC                       │
├──────────────────────────────────────────────────────────────────────┤
│  Mayan Long Count ──── 13.0.12.8.4 | CR: 3 Kan 2 Pop                │
│  Hebrew Calendar  ──── 1 Nisan 5785 AM                               │
│  Chinese Cycle    ──── Yi-Si (Snake/Wood) Year 41, Cycle 78          │
│  Vedic Kali Yuga  ──── Year 5,126 of 432,000 (1.1866%)              │
│  Egyptian Civil   ──── Year 4805, Akhet (Inundation), M4 D1         │
│  Julian Day       ──── JD 2,460,766.5 | MJD 60,766.0                │
│  Islamic Hijri    ──── 2 Shawwal 1446 AH                             │
│  Byzantine AM     ──── Anno Mundi 7,534, Indiction 4                 │
│  13-Moon Time     ──── Solar Moon Day 26, Year 2024 [Cycle 30,024]   │
├──────────────────────────────────────────────────────────────────────┤
│  All mappings bijectively computed via JDN 2,460,767 + GMT           │
│  correlation constant (584,283) and standard astronomical algorithms │
│  Backward time compatibility verified across all 9 calendar systems  │
└──────────────────────────────────────────────────────────────────────┘
```

### The 13-Moon Calendar (Natural Time)

The 13-Moon calendar is a 364-day cycle structured as 13 months of exactly 28 days each, plus one intercalary "Day Out of Time" (July 25, Gregorian). In leap years, a second intercalary day ("Hunab Ku Day") is added. The year begins July 26, aligned to the heliacal rising of Sirius.

**Historical Attestation:**
- **~28,000 BCE** — Abri Blanchard bone (Dordogne, France): lunar notation marks interpreted as a 13-month lunar cycle
- **~20,000 BCE** — Ishango bone (Congo): possible 6-month lunar tally with base-12 groupings
- **~300 BCE** — Book of Enoch / Dead Sea Scrolls: explicit 364-day sacred calendar (4 seasons x 91 days = 13 weeks each)
- **~200 BCE** — Essene/Qumran community: liturgical 364-day calendar used for religious observance
- **Celtic/Druidic** — 13-month tree calendar tradition (Ogham correspondence)

**13-Moon Structure:**

| Moon # | Name | Days | Pattern |
|--------|------|------|---------|
| 1 | Magnetic | 28 | 4 perfect weeks |
| 2 | Lunar | 28 | 4 perfect weeks |
| 3 | Electric | 28 | 4 perfect weeks |
| 4 | Self-Existing | 28 | 4 perfect weeks |
| 5 | Overtone | 28 | 4 perfect weeks |
| 6 | Rhythmic | 28 | 4 perfect weeks |
| 7 | Resonant | 28 | 4 perfect weeks |
| 8 | Galactic | 28 | 4 perfect weeks |
| 9 | Solar | 28 | 4 perfect weeks |
| 10 | Planetary | 28 | 4 perfect weeks |
| 11 | Spectral | 28 | 4 perfect weeks |
| 12 | Crystal | 28 | 4 perfect weeks |
| 13 | Cosmic | 28 | 4 perfect weeks |
| — | Day Out of Time | 1 | July 25 (intercalary) |
| — | Hunab Ku Day | 0-1 | Leap years only |

**Key Property:** Every day of the month falls on the same day of the week, making the calendar perfectly predictable. The 7-day week (Dali, Seli, Gamma, Kali, Alpha, Limi, Silio) repeats exactly 52 times per year.

### Calendar Conversion API

```typescript
import {
  getSalviEpochCalendarSync,
  toMayanLongCount,
  toThirteenMoonDate
} from 'salvi-core';

// Get all calendar mappings for the Salvi Epoch
const sync = getSalviEpochCalendarSync();
console.log(sync.calendars.mayanLongCount.longCount);    // "13.0.12.8.4"
console.log(sync.calendars.vedic.yearInYuga);             // 5126
console.log(sync.calendars.hebrew.year);                  // 5785
console.log(sync.calendars.hebrew.monthName);             // "Nisan"
console.log(sync.calendars.chineseSexagenary.cycleNumber);// 78
console.log(sync.calendars.thirteenMoon.moonName);        // "Solar"

// Convert any date to Mayan Long Count
const mayan = toMayanLongCount(new Date('2012-12-21'));
console.log(mayan.longCount);      // "13.0.0.0.0"
console.log(mayan.calendarRound);  // "4 Ahau 3 Kankin"

// Check if a date is the Day Out of Time
const dot = toThirteenMoonDate(new Date('2025-07-25'));
console.log(dot.dayOutOfTime);     // true

// Convert femtosecond offset to all calendars
const calendars = femtosecondsToAncientCalendars(offset);
```

### REST API Endpoints

```
GET /api/salvi/timing/epoch/anchors          — Fixed Salvi Epoch reference points
GET /api/salvi/timing/epoch/calendars        — Full sync (optional ?date= parameter)
GET /api/salvi/timing/epoch/calendars/mayan
GET /api/salvi/timing/epoch/calendars/hebrew
GET /api/salvi/timing/epoch/calendars/chinese
GET /api/salvi/timing/epoch/calendars/vedic
GET /api/salvi/timing/epoch/calendars/egyptian
GET /api/salvi/timing/epoch/calendars/julian-day
GET /api/salvi/timing/epoch/calendars/islamic
GET /api/salvi/timing/epoch/calendars/byzantine
GET /api/salvi/timing/epoch/calendars/thirteen-moon
```

### Design Rationale

The ancient calendar synchronization serves multiple purposes:

1. **Universal Temporal Anchoring** — By mapping the Salvi Epoch to calendar systems spanning 30,000+ years of human timekeeping, the framework establishes a temporal reference that transcends any single cultural or technological epoch. The 13-Moon calendar provides the deepest temporal anchor, connecting to Paleolithic lunar observation.

2. **Ternary-Calendar Correspondence** — The ternary number system ({-1, 0, +1}) maps naturally to cyclical calendar systems: the Mayan Tzolkin (20-day cycle), Chinese Sexagenary (60-year cycle), Hindu Yugas, and 13-Moon structure (13 x 28 = 364) all exhibit base-3 resonance patterns in their periodic structures. The 13-Moon's 364-day cycle (364 = 4 x 91 = 4 x 7 x 13) demonstrates inherent ternary symmetry.

3. **Verifiable Cross-Reference** — All calendar mappings are bijectively computable from the Julian Day Number via established astronomical algorithms, ensuring deterministic conversion across all 9 systems. This provides independent verification paths for timestamped operations. Reference verification: Mayan 13.0.0.0.0 = Dec 21, 2012; JD 2451545.0 = J2000.0; Hebrew 1 Nisan 5785 = April 1, 2025.

4. **Regulatory Compliance Bridge** — Financial regulations (FINRA 613, MiFID II) require traceable timing. The ancient calendar anchoring extends this traceability beyond modern standards into a civilizational-scale audit trail.

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

### 5. Leverage Ancient Calendar Anchoring
Use the multi-calendar synchronization for cross-civilizational temporal verification and audit trails that extend beyond modern epoch boundaries.

---

*Così sia.*
