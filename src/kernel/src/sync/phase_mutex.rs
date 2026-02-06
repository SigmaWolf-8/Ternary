//! Phase-Safe Mutex Extension
//!
//! Provides a specialized mutex for protecting phase-encrypted data during
//! the recombination process. Enforces timing windows for lock acquisition
//! and ensures that phase components can only be accessed within their
//! cryptographic recombination tolerance.
//!
//! This primitive is critical for the Salvi Framework's split-phase encryption
//! where data halves must be recombined within femtosecond-precision windows.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::cell::UnsafeCell;
use crate::timing::FemtosecondTimestamp;
use crate::phase::EncryptionMode;
use super::{SyncError, SyncResult};

pub struct PhaseMutex<T> {
    locked: AtomicBool,
    lock_timestamp_fs: AtomicU64,
    encryption_mode: EncryptionMode,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for PhaseMutex<T> {}
unsafe impl<T: Send> Sync for PhaseMutex<T> {}

impl<T> PhaseMutex<T> {
    pub fn new(data: T, mode: EncryptionMode) -> Self {
        Self {
            locked: AtomicBool::new(false),
            lock_timestamp_fs: AtomicU64::new(0),
            encryption_mode: mode,
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock_with_timestamp(
        &self,
        timestamp: FemtosecondTimestamp,
    ) -> SyncResult<PhaseMutexGuard<'_, T>> {
        match self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                self.lock_timestamp_fs.store(
                    timestamp.femtoseconds as u64,
                    Ordering::Release,
                );
                Ok(PhaseMutexGuard { mutex: self })
            }
            Err(_) => {
                let lock_time = self.lock_timestamp_fs.load(Ordering::Acquire);
                let elapsed = timestamp.femtoseconds.saturating_sub(lock_time as u128);
                let tolerance = self.encryption_mode.recombination_tolerance_fs();

                if elapsed > tolerance {
                    Err(SyncError::PhaseWindowExpired {
                        elapsed_fs: elapsed,
                        tolerance_fs: tolerance,
                    })
                } else {
                    Err(SyncError::WouldBlock)
                }
            }
        }
    }

    pub fn try_lock_within_window(
        &self,
        timestamp: FemtosecondTimestamp,
        reference_timestamp: FemtosecondTimestamp,
    ) -> SyncResult<PhaseMutexGuard<'_, T>> {
        let time_diff = if timestamp.femtoseconds > reference_timestamp.femtoseconds {
            timestamp.femtoseconds - reference_timestamp.femtoseconds
        } else {
            reference_timestamp.femtoseconds - timestamp.femtoseconds
        };

        let tolerance = self.encryption_mode.recombination_tolerance_fs();
        if time_diff >= tolerance {
            return Err(SyncError::PhaseWindowExpired {
                elapsed_fs: time_diff,
                tolerance_fs: tolerance,
            });
        }

        self.lock_with_timestamp(timestamp)
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn lock_age_fs(&self, current: FemtosecondTimestamp) -> Option<u128> {
        if !self.is_locked() {
            return None;
        }
        let lock_time = self.lock_timestamp_fs.load(Ordering::Acquire) as u128;
        Some(current.femtoseconds.saturating_sub(lock_time))
    }

    pub fn is_window_expired(&self, current: FemtosecondTimestamp) -> bool {
        match self.lock_age_fs(current) {
            Some(age) => age > self.encryption_mode.recombination_tolerance_fs(),
            None => false,
        }
    }

    pub fn encryption_mode(&self) -> EncryptionMode {
        self.encryption_mode
    }

    fn unlock(&self) {
        self.lock_timestamp_fs.store(0, Ordering::Release);
        self.locked.store(false, Ordering::Release);
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

pub struct PhaseMutexGuard<'a, T> {
    mutex: &'a PhaseMutex<T>,
}

impl<'a, T> core::ops::Deref for PhaseMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for PhaseMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for PhaseMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_mutex_creation() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced);
        assert!(!pmx.is_locked());
        assert_eq!(pmx.encryption_mode(), EncryptionMode::Balanced);
    }

    #[test]
    fn test_phase_mutex_lock_with_timestamp() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced);
        let ts = FemtosecondTimestamp::new(1_000_000);

        let guard = pmx.lock_with_timestamp(ts).unwrap();
        assert!(pmx.is_locked());
        assert_eq!(*guard, 42);

        drop(guard);
        assert!(!pmx.is_locked());
    }

    #[test]
    fn test_phase_mutex_modify() {
        let pmx = PhaseMutex::new(0, EncryptionMode::Performance);
        let ts = FemtosecondTimestamp::new(1_000);

        {
            let mut guard = pmx.lock_with_timestamp(ts).unwrap();
            *guard = 99;
        }

        let ts2 = FemtosecondTimestamp::new(2_000);
        let guard = pmx.lock_with_timestamp(ts2).unwrap();
        assert_eq!(*guard, 99);
    }

    #[test]
    fn test_phase_mutex_window_check() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced); // 100fs tolerance
        let ts1 = FemtosecondTimestamp::new(1_000);
        let ts2 = FemtosecondTimestamp::new(1_050); // within window
        let ts3 = FemtosecondTimestamp::new(2_000); // outside window

        let result = pmx.try_lock_within_window(ts2, ts1);
        assert!(result.is_ok());
        drop(result);

        let result = pmx.try_lock_within_window(ts3, ts1);
        assert!(result.is_err());
    }

    #[test]
    fn test_phase_mutex_high_security_tighter_window() {
        let pmx = PhaseMutex::new(42, EncryptionMode::HighSecurity); // 50fs tolerance
        let ts1 = FemtosecondTimestamp::new(1_000);
        let ts2 = FemtosecondTimestamp::new(1_060); // > 50fs

        let result = pmx.try_lock_within_window(ts2, ts1);
        assert!(result.is_err());
    }

    #[test]
    fn test_phase_mutex_performance_wider_window() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Performance); // 500fs tolerance
        let ts1 = FemtosecondTimestamp::new(1_000);
        let ts2 = FemtosecondTimestamp::new(1_400); // < 500fs

        let result = pmx.try_lock_within_window(ts2, ts1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_phase_mutex_lock_age() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced);
        let ts1 = FemtosecondTimestamp::new(1_000);

        let _guard = pmx.lock_with_timestamp(ts1).unwrap();
        let ts2 = FemtosecondTimestamp::new(1_050);
        let age = pmx.lock_age_fs(ts2);
        assert_eq!(age, Some(50));
    }

    #[test]
    fn test_phase_mutex_lock_age_unlocked() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced);
        let ts = FemtosecondTimestamp::new(1_000);
        assert_eq!(pmx.lock_age_fs(ts), None);
    }

    #[test]
    fn test_phase_mutex_window_expired() {
        let pmx = PhaseMutex::new(42, EncryptionMode::HighSecurity); // 50fs
        let ts1 = FemtosecondTimestamp::new(1_000);

        let _guard = pmx.lock_with_timestamp(ts1).unwrap();

        let ts_within = FemtosecondTimestamp::new(1_040);
        assert!(!pmx.is_window_expired(ts_within));

        let ts_expired = FemtosecondTimestamp::new(1_060);
        assert!(pmx.is_window_expired(ts_expired));
    }

    #[test]
    fn test_phase_mutex_into_inner() {
        let pmx = PhaseMutex::new(vec![1, 2, 3], EncryptionMode::Balanced);
        let data = pmx.into_inner();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_phase_mutex_contention() {
        let pmx = PhaseMutex::new(42, EncryptionMode::Balanced);
        let ts1 = FemtosecondTimestamp::new(1_000);
        let ts2 = FemtosecondTimestamp::new(1_050);

        let _guard = pmx.lock_with_timestamp(ts1).unwrap();
        let result = pmx.lock_with_timestamp(ts2);
        assert!(result.is_err());
    }
}
