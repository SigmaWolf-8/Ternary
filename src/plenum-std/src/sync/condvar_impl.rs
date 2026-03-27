use super::mutex_impl::{MutexGuard, LockResult};
use core::sync::atomic::{AtomicU64, Ordering};

pub struct Condvar {
    waiters: AtomicU64,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            waiters: AtomicU64::new(0),
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> LockResult<MutexGuard<'a, T>> {
        self.waiters.fetch_add(1, Ordering::Relaxed);
        let mutex_ref = guard.mutex_ref();
        drop(guard);
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
        self.waiters.fetch_sub(1, Ordering::Relaxed);
        mutex_ref.lock()
    }

    pub fn notify_one(&self) {
        let _ = self.waiters.load(Ordering::Relaxed);
    }

    pub fn notify_all(&self) {
        let _ = self.waiters.load(Ordering::Relaxed);
    }
}
