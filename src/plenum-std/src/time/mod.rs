pub use core::time::Duration;

use core::sync::atomic::{AtomicU64, Ordering};

static MONOTONIC_NANOS: AtomicU64 = AtomicU64::new(0);

type ClockSourceFn = fn() -> u64;
static mut CLOCK_SOURCE: Option<ClockSourceFn> = None;

pub fn register_clock_source(source: ClockSourceFn) {
    unsafe {
        CLOCK_SOURCE = Some(source);
    }
}

fn read_monotonic_nanos() -> u64 {
    if let Some(source) = unsafe { CLOCK_SOURCE } {
        return source();
    }
    MONOTONIC_NANOS.fetch_add(1_000, Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            nanos: read_monotonic_nanos(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        let now = read_monotonic_nanos();
        let diff = now.saturating_sub(self.nanos);
        Duration::from_nanos(diff)
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let diff = self.nanos.saturating_sub(earlier.nanos);
        Duration::from_nanos(diff)
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.nanos
            .checked_sub(earlier.nanos)
            .map(Duration::from_nanos)
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.duration_since(earlier)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        let nanos_add = duration.as_nanos() as u64;
        self.nanos.checked_add(nanos_add).map(|n| Instant { nanos: n })
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        let nanos_sub = duration.as_nanos() as u64;
        self.nanos.checked_sub(nanos_sub).map(|n| Instant { nanos: n })
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, dur: Duration) -> Instant {
        self.checked_add(dur)
            .expect("overflow when adding duration to instant")
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, dur: Duration) -> Instant {
        self.checked_sub(dur)
            .expect("overflow when subtracting duration from instant")
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl core::ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, dur: Duration) {
        *self = *self + dur;
    }
}

impl core::ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, dur: Duration) {
        *self = *self - dur;
    }
}

const UNIX_EPOCH_OFFSET_SECS: u64 = 1_743_465_600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    secs: u64,
    nanos: u32,
}

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = SystemTime { secs: 0, nanos: 0 };

    pub fn now() -> Self {
        let ns = read_monotonic_nanos();
        let secs = UNIX_EPOCH_OFFSET_SECS + ns / 1_000_000_000;
        let sub_nanos = (ns % 1_000_000_000) as u32;
        SystemTime {
            secs,
            nanos: sub_nanos,
        }
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        if self.secs > earlier.secs
            || (self.secs == earlier.secs && self.nanos >= earlier.nanos)
        {
            let secs = self.secs - earlier.secs;
            let nanos = if self.nanos >= earlier.nanos {
                self.nanos - earlier.nanos
            } else {
                return Ok(Duration::new(
                    secs - 1,
                    1_000_000_000 + self.nanos - earlier.nanos,
                ));
            };
            Ok(Duration::new(secs, nanos))
        } else {
            Err(SystemTimeError(Duration::new(
                earlier.secs - self.secs,
                0,
            )))
        }
    }

    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(*self)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        let secs = self.secs.checked_add(duration.as_secs())?;
        let nanos = self.nanos + duration.subsec_nanos();
        if nanos >= 1_000_000_000 {
            Some(SystemTime {
                secs: secs.checked_add(1)?,
                nanos: nanos - 1_000_000_000,
            })
        } else {
            Some(SystemTime { secs, nanos })
        }
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        let secs = self.secs.checked_sub(duration.as_secs())?;
        if self.nanos >= duration.subsec_nanos() {
            Some(SystemTime {
                secs,
                nanos: self.nanos - duration.subsec_nanos(),
            })
        } else {
            Some(SystemTime {
                secs: secs.checked_sub(1)?,
                nanos: 1_000_000_000 + self.nanos - duration.subsec_nanos(),
            })
        }
    }
}

impl core::ops::Add<Duration> for SystemTime {
    type Output = SystemTime;
    fn add(self, dur: Duration) -> SystemTime {
        self.checked_add(dur)
            .expect("overflow when adding duration to system time")
    }
}

impl core::ops::Sub<Duration> for SystemTime {
    type Output = SystemTime;
    fn sub(self, dur: Duration) -> SystemTime {
        self.checked_sub(dur)
            .expect("overflow when subtracting duration from system time")
    }
}

pub const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct SystemTimeError(Duration);

impl SystemTimeError {
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl core::fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "second time provided was later than self")
    }
}
