//! Mutex Implementation
//!
//! Provides a mutual exclusion lock with ternary security mode awareness.
//! Built on top of the ticket spinlock for `no_std` environments.
//! Supports security mode restrictions where lock acquisition can be
//! gated by the caller's security privilege level.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::cell::UnsafeCell;
use crate::memory::SecurityMode;
use super::{LockState, SyncError, SyncResult};

pub struct Mutex<T> {
    locked: AtomicBool,
    minimum_mode: AtomicU8,
    data: UnsafeCell<T>,
    poisoned: AtomicBool,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            minimum_mode: AtomicU8::new(1), // ModeZero by default (accessible by all)
            data: UnsafeCell::new(data),
            poisoned: AtomicBool::new(false),
        }
    }

    pub fn with_security_mode(data: T, mode: SecurityMode) -> Self {
        Self {
            locked: AtomicBool::new(false),
            minimum_mode: AtomicU8::new(mode.access_level()),
            data: UnsafeCell::new(data),
            poisoned: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> SyncResult<MutexGuard<'_, T>> {
        if self.poisoned.load(Ordering::Relaxed) {
            return Err(SyncError::Poisoned);
        }

        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }

        Ok(MutexGuard { mutex: self })
    }

    pub fn lock_with_mode(&self, caller_mode: SecurityMode) -> SyncResult<MutexGuard<'_, T>> {
        let required = self.minimum_mode.load(Ordering::Relaxed);
        if caller_mode.access_level() < required {
            return Err(SyncError::SecurityViolation);
        }

        self.lock()
    }

    pub fn try_lock(&self) -> SyncResult<MutexGuard<'_, T>> {
        if self.poisoned.load(Ordering::Relaxed) {
            return Err(SyncError::Poisoned);
        }

        match self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(MutexGuard { mutex: self }),
            Err(_) => Err(SyncError::WouldBlock),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Relaxed)
    }

    pub fn state(&self) -> LockState {
        if self.locked.load(Ordering::Relaxed) {
            LockState::Locked
        } else {
            LockState::Unlocked
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_creation() {
        let mutex = Mutex::new(42);
        assert!(!mutex.is_locked());
        assert!(!mutex.is_poisoned());
        assert_eq!(mutex.state(), LockState::Unlocked);
    }

    #[test]
    fn test_mutex_lock_unlock() {
        let mutex = Mutex::new(42);
        {
            let guard = mutex.lock().unwrap();
            assert!(mutex.is_locked());
            assert_eq!(*guard, 42);
        }
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_mutex_modify() {
        let mutex = Mutex::new(0);
        {
            let mut guard = mutex.lock().unwrap();
            *guard = 100;
        }
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_mutex_try_lock() {
        let mutex = Mutex::new(42);
        let guard = mutex.try_lock().unwrap();
        assert_eq!(*guard, 42);

        let result = mutex.try_lock();
        assert!(result.is_err());
    }

    #[test]
    fn test_mutex_security_mode_access() {
        let mutex = Mutex::with_security_mode(42, SecurityMode::ModeOne);

        let result = mutex.lock_with_mode(SecurityMode::ModePhi);
        assert!(result.is_ok());
        drop(result);

        let result = mutex.lock_with_mode(SecurityMode::ModeOne);
        assert!(result.is_ok());
        drop(result);

        let result = mutex.lock_with_mode(SecurityMode::ModeZero);
        assert!(result.is_err());
    }

    #[test]
    fn test_mutex_security_mode_phi_guards() {
        let mutex = Mutex::with_security_mode(vec![1, 2, 3], SecurityMode::ModePhi);

        let result = mutex.lock_with_mode(SecurityMode::ModeOne);
        assert!(result.is_err());

        let result = mutex.lock_with_mode(SecurityMode::ModePhi);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutex_into_inner() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let data = mutex.into_inner();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_mutex_state_transitions() {
        let mutex = Mutex::new(0);
        assert_eq!(mutex.state(), LockState::Unlocked);

        let guard = mutex.lock().unwrap();
        assert_eq!(mutex.state(), LockState::Locked);

        drop(guard);
        assert_eq!(mutex.state(), LockState::Unlocked);
    }
}
