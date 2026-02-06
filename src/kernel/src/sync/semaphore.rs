//! Counting Semaphore Implementation
//!
//! Provides a counting semaphore for managing access to a finite pool of
//! resources. Supports both binary (mutex-like) and counting configurations.
//! Used for ternary compute buffer pools and phase encryption channel management.

use core::sync::atomic::{AtomicUsize, Ordering};
use super::{SyncError, SyncResult};

pub struct Semaphore {
    count: AtomicUsize,
    max_count: usize,
}

impl Semaphore {
    pub const fn new(initial_count: usize, max_count: usize) -> Self {
        Self {
            count: AtomicUsize::new(initial_count),
            max_count,
        }
    }

    pub const fn binary() -> Self {
        Self::new(1, 1)
    }

    pub fn acquire(&self) -> SyncResult<SemaphorePermit<'_>> {
        loop {
            let current = self.count.load(Ordering::Relaxed);
            if current == 0 {
                core::hint::spin_loop();
                continue;
            }

            match self.count.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(SemaphorePermit { semaphore: self }),
                Err(_) => continue,
            }
        }
    }

    pub fn try_acquire(&self) -> SyncResult<SemaphorePermit<'_>> {
        let current = self.count.load(Ordering::Relaxed);
        if current == 0 {
            return Err(SyncError::WouldBlock);
        }

        match self.count.compare_exchange(
            current,
            current - 1,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(SemaphorePermit { semaphore: self }),
            Err(_) => Err(SyncError::WouldBlock),
        }
    }

    fn release(&self) {
        let mut current = self.count.load(Ordering::Relaxed);
        loop {
            if current >= self.max_count {
                break;
            }
            match self.count.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn available(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn max_count(&self) -> usize {
        self.max_count
    }

    pub fn is_available(&self) -> bool {
        self.count.load(Ordering::Relaxed) > 0
    }
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        self.semaphore.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semaphore_creation() {
        let sem = Semaphore::new(3, 3);
        assert_eq!(sem.available(), 3);
        assert_eq!(sem.max_count(), 3);
        assert!(sem.is_available());
    }

    #[test]
    fn test_semaphore_binary() {
        let sem = Semaphore::binary();
        assert_eq!(sem.available(), 1);
        assert_eq!(sem.max_count(), 1);
    }

    #[test]
    fn test_semaphore_acquire_release() {
        let sem = Semaphore::new(2, 2);

        let p1 = sem.try_acquire().unwrap();
        assert_eq!(sem.available(), 1);

        let p2 = sem.try_acquire().unwrap();
        assert_eq!(sem.available(), 0);

        drop(p1);
        assert_eq!(sem.available(), 1);

        drop(p2);
        assert_eq!(sem.available(), 2);
    }

    #[test]
    fn test_semaphore_exhaustion() {
        let sem = Semaphore::new(1, 1);
        let _p = sem.try_acquire().unwrap();
        let result = sem.try_acquire();
        assert!(result.is_err());
    }

    #[test]
    fn test_semaphore_multiple_permits() {
        let sem = Semaphore::new(5, 5);
        let mut permits = alloc::vec::Vec::new();

        for _ in 0..5 {
            permits.push(sem.try_acquire().unwrap());
        }

        assert_eq!(sem.available(), 0);
        assert!(!sem.is_available());

        permits.clear();
        assert_eq!(sem.available(), 5);
        assert!(sem.is_available());
    }

    #[test]
    fn test_semaphore_no_overflow() {
        let sem = Semaphore::new(1, 1);
        let permit = sem.try_acquire().unwrap();
        drop(permit);
        assert_eq!(sem.available(), 1);

        sem.release();
        assert_eq!(sem.available(), 1); // Should not exceed max
    }
}
