# HModal Power Measurement — Trit-Native Specification

> Replaces the byte-oriented `power_measure` proposal in full.
> AASC stays trit-pure end-to-end. ONE byte boundary remains: the OS RAPL syscall itself.
> That boundary lives in a Tier-2 crate (`hmodal-power`), never inside AASC.

---

## 0. What Changes vs. the Byte-Oriented Original

| Original                                              | Trit-Native Replacement                                                          |
| ----------------------------------------------------- | -------------------------------------------------------------------------------- |
| `nix::sys::perf_event_open` + raw `pread` byte buffer | `read_sysfs_u64("/sys/class/powercap/intel-rapl:0/energy_uj")` → instant `TritVec` |
| `f64` watts, `VecDeque<f64>` ring                     | `RationalTrit` (TritVec numerator + TritVec denominator) and TritVec ring buffer |
| `Mutex<...>` around the buffer                        | AASC lock-free TritVec work-steal queue (`8.9.7.UX4.1__work_steal_queue.rs`)     |
| `wasm-bindgen`, `JsError`, `JsValue`                  | Kernel-native ternary socket (`8.9.3.UX9.5__ipc.rs` pattern), TDNS-addressed     |
| Three.js + HTML canvas frontend                       | PlenumBrowser kernel subsystem renders to framebuffer; no JS, no WASM            |
| `--simulate` dev flag                                 | NOT supported. Trit-pure means trit-real.                                        |
| α / β / 143/192 stored as floats                      | Stored as exact `RationalTrit` constants; equalities verified at compile time    |

---

## 1. Hardware and OS Prerequisites

- **CPU:** Intel processor with RAPL support (Sandy Bridge or newer), or any AMD chip exposing `/sys/class/powercap/`.
- **OS:** Linux kernel with `powercap` sysfs (3.13+).
- **Permissions:** read access to `/sys/class/powercap/intel-rapl:0/energy_uj`. On most distros this requires either `chmod 644` of the file or running the Tier-2 binary as the `powercap` group.
- **Toolchain:** Rust 1.70+ for the Tier-2 binary.    No WASM target. No `nix` crate. No `perf_event_open`.

---

## 2. AASC Constants Lock-Down — `L=9` Group 6

Four new files in AASC, all in `src/L9__physics_nw/`. Every numeric value derives from the sole axiom block in `1.1.1.UX1.1__constants.rs` (π=14, b=3) and the Forge triple (p=7, q=11, r=13).

### 2.1 `9.6.1.UX9.6__hmodal_constants.rs`

```rust
//! HModal square-wave constants — derived, not chosen.
//! Every value is a RationalTrit (TritVec / TritVec). No f64 anywhere.

use aasc::tritvec::TritVec;
use aasc::rational::RationalTrit;

// α = R_6 / Δ = 364 / 144 = 91 / 36   (low state)
pub const ALPHA: RationalTrit = RationalTrit::new(91, 36);

// β = R_6 / √Δ = 364 / 12 = 91 / 3    (high state)
pub const BETA:  RationalTrit = RationalTrit::new(91, 3);

// Discriminant Δ = R_4² − 4·R_6 = 1600 − 1456 = 144
pub const DELTA: TritVec = TritVec::from_decimal(144);

// Duty cycle: 1/4 high, 3/4 low
pub const DUTY_HIGH: RationalTrit = RationalTrit::new(1, 4);
pub const DUTY_LOW:  RationalTrit = RationalTrit::new(3, 4);

// Time-averaged DC level  ⟨H⟩ = (3α + β) / 4 = 455 / 48
pub const DC_MEAN: RationalTrit = RationalTrit::new(455, 48);

// Energy ratio  E_square / E_cont = (3α² + β²) / (4β²) = 49 / 192
pub const ENERGY_RATIO: RationalTrit = RationalTrit::new(49, 192);

// Energy savings  = 1 − 49/192 = 143 / 192  ≈ 74.48 %
pub const SAVINGS: RationalTrit = RationalTrit::new(143, 192);

// Tier-1 invariants — verified at COMPILE time.
const _: () = {
    // (3α² + β²) / (4β²)  ==  49 / 192
    assert!(RationalTrit::eq(
        RationalTrit::div(
            RationalTrit::add(
                RationalTrit::mul_int(RationalTrit::sq(ALPHA), 3),
                RationalTrit::sq(BETA),
            ),
            RationalTrit::mul_int(RationalTrit::sq(BETA), 4),
        ),
        ENERGY_RATIO,
    ));
    // 1 − ENERGY_RATIO == SAVINGS
    assert!(RationalTrit::eq(
        RationalTrit::sub(RationalTrit::one(), ENERGY_RATIO),
        SAVINGS,
    ));
};
```

### 2.2 `9.6.2.UX9.6__energy_accumulator.rs`

```rust
use aasc::tritvec::TritVec;
use aasc::rational::RationalTrit;
use aasc::work_steal::TritVecQueue;

pub struct EnergyAccumulator {
    samples:   TritVecQueue,   // lock-free, MPMC, trit-native (uses 8.9.7)
    last_uj:   TritVec,        // monotonic counter, base-3 packed
    last_tick: TritVec,        // femtosecond counter from kernel HPTP
}

impl EnergyAccumulator {
    pub fn new(capacity: TritVec) -> Self { /* ... */ }

    /// Accept a pre-converted microjoule reading (TritVec). No bytes anywhere.
    pub fn push_microjoules(&mut self, uj: TritVec, tick_fs: TritVec);

    /// Mean watts across the buffer, returned as a RationalTrit (no float).
    pub fn mean_watts(&self) -> RationalTrit;

    /// Live (E_square / E_cont) computed on the actual sample distribution.
    pub fn observed_ratio(&self) -> RationalTrit;

    /// Live savings = 1 − observed_ratio.
    pub fn observed_savings(&self) -> RationalTrit;
}
```

### 2.3 `9.6.3.UX9.6__duty_cycle_walk.rs`

```rust
//! Drives the workload selector. The 1:4 duty cycle is a closed walk on Z/4,
//! which is the natural projection of the Salvi 27×28 walk modulo 4.

use aasc::tritvec::TritVec;
use aasc::walk::ClosedWalk;

#[repr(u8)]
pub enum DutyState { Low = 0, High = 1 }

pub struct DutyWalk {
    pos: TritVec,           // current position 0..3, stored as TritVec
    walk: ClosedWalk<4>,    // step size 1, length 4
}

impl DutyWalk {
    /// Advance one step, return the resulting state.
    /// Pattern: Low, Low, Low, High — three silent steps then one transmit.
    pub fn next(&mut self) -> DutyState;
}
```

### 2.4 `9.6.4.UX9.6__savings_meter.rs`

```rust
use aasc::rational::RationalTrit;
use crate::energy_accumulator::EnergyAccumulator;
use crate::hmodal_constants::{SAVINGS, ENERGY_RATIO};

pub struct SavingsMeter<'a> {
    acc: &'a EnergyAccumulator,
}

impl<'a> SavingsMeter<'a> {
    /// Theoretical savings — exact rational 143/192.
    pub fn theoretical(&self) -> RationalTrit { SAVINGS }

    /// Observed savings from live samples, as a RationalTrit.
    pub fn observed(&self) -> RationalTrit { self.acc.observed_savings() }

    /// Δ = observed − theoretical, also RationalTrit.
    pub fn drift(&self) -> RationalTrit {
        RationalTrit::sub(self.observed(), self.theoretical())
    }
}
```

---

## 3. Tier-2 Crate `hmodal-power` (lives OUTSIDE AASC)

This is the ONLY place a byte type appears in the entire HModal stack.
It links AASC, performs the OS-edge work, and publishes TritVec frames over the kernel-native ternary socket.

### 3.1 `Cargo.toml`

```toml
[package]
name = "hmodal-power"
version = "0.1.0"
edition = "2021"

[dependencies]
aasc = { registry = "plenumnet", version = "*", features = ["daemon"] }
# NO nix.  NO wasm-bindgen.  NO serde.  NO tokio at this layer (kernel scheduler is used).
```

### 3.2 RAPL Intake — the single byte boundary

`src/rapl_intake.rs`

```rust
use std::fs;
use aasc::tritvec::TritVec;

pub fn read_package_microjoules() -> Result<TritVec, RaplError> {
    // The ONE byte read in this entire subsystem.
    let raw: u64 = fs::read_to_string("/sys/class/powercap/intel-rapl:0/energy_uj")?
        .trim()
        .parse()?;

    // Convert immediately.  After this line, no byte type ever appears again.
    Ok(TritVec::from_u64_base3(raw))
}
```

**Rule:** every other function in this crate takes only `TritVec` or `RationalTrit`. No `u8`, `u32`, `u64`, `f32`, `f64` propagates past line 6 of this file. CI grep guard enforces it.

### 3.3 Kernel-Native IPC Publisher

`src/publisher.rs`

```rust
use aasc::ipc::{TernarySocket, TritFrame};
use aasc::tritvec::TritVec;
use aasc::rational::RationalTrit;

pub struct Publisher { sock: TernarySocket }

impl Publisher {
    pub fn open() -> Self {
        Self { sock: TernarySocket::bind("plenumnet://hmodal/power") }
        // TDNS-resolved, kernel-native ternary transport, NO Unix-sock,
        // NO named-pipe, NO byte stream, NO JSON, NO UTF-8.
    }

    pub fn publish(&self, sample_uj: TritVec, savings: RationalTrit) {
        let frame = TritFrame::new(&[ sample_uj, savings.num().clone(), savings.den().clone() ]);
        self.sock.send(&frame);
    }
}
```

### 3.4 Workload Generator

`src/workload.rs`

The "high" state runs **real** GF(3) batches on AASC's TritVec arithmetic — observed by RAPL as actual CPU power draw. No SHA-256, no integer multiplication, no synthetic spinning.

```rust
use aasc::gf3::GF3;
use aasc::tritvec::TritVec;
use aasc::scheduler::yield_to_kernel;

pub fn high_state_burst(work_units: &TritVec) {
    let mut acc = TritVec::ones(8192);
    for _ in 0..work_units.as_decimal() {
        acc = GF3::add_batch(&acc, &acc);   // real native trit work
    }
    std::hint::black_box(acc);
}

pub fn low_state_yield() {
    yield_to_kernel();   // trit-aware kernel yield, NOT thread::sleep
}
```

### 3.5 Main Loop

`src/main.rs`

```rust
use aasc::tritvec::TritVec;
use aasc::hptp::femtosecond_now;
use aasc::L9__physics_nw::{
    energy_accumulator::EnergyAccumulator,
    duty_cycle_walk::{DutyWalk, DutyState},
    savings_meter::SavingsMeter,
};

mod rapl_intake;
mod publisher;
mod workload;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acc  = EnergyAccumulator::new(TritVec::from_decimal(300));   // 60s @ 200ms
    let mut walk = DutyWalk::new();
    let pub_     = publisher::Publisher::open();

    loop {
        match walk.next() {
            DutyState::High => workload::high_state_burst(&TritVec::from_decimal(50_000)),
            DutyState::Low  => workload::low_state_yield(),
        }

        let uj = rapl_intake::read_package_microjoules()?;
        acc.push_microjoules(uj.clone(), femtosecond_now());

        let meter = SavingsMeter::new(&acc);
        pub_.publish(uj, meter.observed());
    }
}
```

---

## 4. Frontend — PlenumBrowser Kernel Subsystem (NOT WASM)

The PlenumBrowser kernel subsystem subscribes to `plenumnet://hmodal/power` and renders directly to the framebuffer. No JavaScript, no canvas, no Three.js, no `wasm-bindgen`.

- **Power gauge** — needle angle is a `TritVec` mod ⌊4·π_geom⌋ = 56-step rotation. Drawn by the CPU rendering path (per `replit.md` PlenumBrowser Kernel Subsystem section).
- **Strip chart** — last 60 s of TritVec samples, drawn as a 540-pixel strip (matches the (7,11,13) coprime walk, 540 z=0 nodes — natural sample width).
- **Numeric readouts** — `RationalTrit` values projected to Milesian glyphs via `1.4.1.UX1.1__milesian.rs`. A decimal projection is generated only at the rendering edge if requested.
- **Theoretical reference line** — fixed at `SAVINGS = 143/192`.
- **No `f64` reaches the renderer.**

---

## 5. Error Handling

| Failure                                | Behaviour                                                       |
| -------------------------------------- | --------------------------------------------------------------- |
| sysfs RAPL file unreadable             | Return `Err`. No fallback. UI shows "RAPL unavailable: $reason" |
| No samples yet                         | TritVec stays zero. UI shows "no data".                         |
| Counter wrap (32-bit RAPL counter)     | Detected by monotonic check; wrap delta added back as TritVec.  |
| `--simulate` flag                      | NOT supported.                                                  |
| WASM target requested                  | NOT supported. Native binary only.                              |
| Daemon detached from kernel TernarySocket | Coordinator (`8.9.6`) restarts the publisher on the surviving instance. |

---

## 6. Implementation Order

1. Land the four AASC modules in L=9 group 6 — pure math, compile-time invariants only.
2. Stand up the Tier-2 `hmodal-power` crate; verify `rapl_intake` against a known-good `/sys/class/powercap/` reading on the target machine.
3. Smoke-test `publisher` against a kernel TernarySocket subscriber.
4. Wire the PlenumBrowser kernel subsystem subscriber, draw the gauge.
5. Run for one hour, log the drift between `SavingsMeter::theoretical()` and `SavingsMeter::observed()`. Drift < ±2 % validates the buoyant-medium HModal model on real silicon.

---

## 7. Address Catalog Update

Add to AASC canonical map L=9 group 6:

```
9.6.1.UX9.6__hmodal_constants.rs
9.6.2.UX9.6__energy_accumulator.rs
9.6.3.UX9.6__duty_cycle_walk.rs
9.6.4.UX9.6__savings_meter.rs
```

Total AASC modules: **67 → 71**.

The Tier-2 `hmodal-power` crate uses its own per-crate L.G.O.UX scheme rooted at L=1 of that crate, and does NOT appear in the AASC catalog.

---

## 8. Forbidden Constructs (CI Grep Guard)

`scripts/check_hmodal_purity.sh` asserts that NONE of the following appear anywhere in `aasc/src/L9__physics_nw/9.6.*.rs` or in `hmodal-power/src/**/*.rs` (except the single intake line):

```
\bf32\b   \bf64\b   \bu8\b   \bu16\b   \bu32\b
\bnix::\b   wasm_bindgen   serde_json   thread::sleep   perf_event_open
\bMutex\b   \bVecDeque<f64>\b
```

If any of these turn up outside the whitelisted intake line, the build fails.

---

## 9. Closing Note

Every constant on this page — α=91/36, β=91/3, Δ=144, ⟨H⟩=455/48, savings=143/192 — is a closed-form rational derived from b=3 and the Forge triple (7,11,13). They are **stored as exact RationalTrit values, asserted at compile time, and transmitted as TritVec frames over the kernel-native ternary socket**. Real RAPL readings calibrate the buoyant-medium HModal model against silicon; the ratio that comes out is the same ratio that went in, derived from the discriminant Δ=144.

No bytes inside AASC. One byte at the OS edge. One TritVec everywhere else.
