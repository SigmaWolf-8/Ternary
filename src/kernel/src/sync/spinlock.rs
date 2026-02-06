//! Ticket Spinlock Implementation
//!
//! Implements a ticket-based spinlock providing FIFO fairness guarantees.
//! Uses atomic operations for lock-free contention management.
//! Suitable for short critical sections in interrupt handlers and
//! ternary compute operations.

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

pub struct Spinlock<T> {
    next_ticket: AtomicUsize,
    now_serving: AtomicUsize,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            next_ticket: AtomicUsize::new(0),
            now_serving: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        let ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);

        while self.now_serving.load(Ordering::Acquire) != ticket {
            core::hint::spin_loop();
        }

        SpinlockGuard { lock: self }
    }

    pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
        let current = self.now_serving.load(Ordering::Relaxed);
        let result = self.next_ticket.compare_exchange(
            current,
            current + 1,
            Ordering::Acquire,
            Ordering::Relaxed,
        );

        match result {
            Ok(_) => Some(SpinlockGuard { lock: self }),
            Err(_) => None,
        }
    }

    pub fn is_locked(&self) -> bool {
        let serving = self.now_serving.load(Ordering::Relaxed);
        let next = self.next_ticket.load(Ordering::Relaxed);
        serving != next
    }

    fn unlock(&self) {
        self.now_serving.fetch_add(1, Ordering::Release);
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<'a, T> core::ops::Deref for SpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinlock_creation() {
        let lock = Spinlock::new(42);
        assert!(!lock.is_locked());
    }

    #[test]
    fn test_spinlock_lock_unlock() {
        let lock = Spinlock::new(42);
        {
            let guard = lock.lock();
            assert!(lock.is_locked());
            assert_eq!(*guard, 42);
        }
        assert!(!lock.is_locked());
    }

    #[test]
    fn test_spinlock_modify() {
        let lock = Spinlock::new(0);
        {
            let mut guard = lock.lock();
            *guard = 100;
        }
        {
            let guard = lock.lock();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_spinlock_try_lock_success() {
        let lock = Spinlock::new(42);
        let guard = lock.try_lock();
        assert!(guard.is_some());
        assert_eq!(*guard.unwrap(), 42);
    }

    #[test]
    fn test_spinlock_try_lock_fail() {
        let lock = Spinlock::new(42);
        let _guard = lock.lock();
        let result = lock.try_lock();
        assert!(result.is_none());
    }

    #[test]
    fn test_spinlock_into_inner() {
        let lock = Spinlock::new(vec![1, 2, 3]);
        let data = lock.into_inner();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_spinlock_get_mut() {
        let mut lock = Spinlock::new(42);
        *lock.get_mut() = 99;
        let guard = lock.lock();
        assert_eq!(*guard, 99);
    }

    #[test]
    fn test_spinlock_sequential_access() {
        let lock = Spinlock::new(0u32);

        for i in 0..100 {
            let mut guard = lock.lock();
            *guard = i;
        }

        let guard = lock.lock();
        assert_eq!(*guard, 99);
    }
}
