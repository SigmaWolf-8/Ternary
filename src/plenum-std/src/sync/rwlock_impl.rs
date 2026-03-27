use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicIsize, AtomicBool, Ordering};
use super::mutex_impl::PoisonError;

pub type LockResult<T> = Result<T, PoisonError<T>>;

pub struct RwLock<T: ?Sized> {
    state: AtomicIsize,
    poisoned: AtomicBool,
    writer_locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(t: T) -> Self {
        Self {
            state: AtomicIsize::new(0),
            poisoned: AtomicBool::new(false),
            writer_locked: AtomicBool::new(false),
            data: UnsafeCell::new(t),
        }
    }

    pub fn into_inner(self) -> LockResult<T> {
        let data = self.data.into_inner();
        if self.poisoned.load(Ordering::Relaxed) {
            Err(PoisonError::new(data))
        } else {
            Ok(data)
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        loop {
            if self.writer_locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
                continue;
            }
            let current = self.state.load(Ordering::Relaxed);
            if current < 0 {
                core::hint::spin_loop();
                continue;
            }
            if self
                .state
                .compare_exchange_weak(current, current + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                let guard = RwLockReadGuard { lock: self };
                if self.poisoned.load(Ordering::Relaxed) {
                    return Err(PoisonError::new(guard));
                }
                return Ok(guard);
            }
        }
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        while self
            .writer_locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        while self
            .state
            .compare_exchange_weak(0, -1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        let guard = RwLockWriteGuard { lock: self };
        if self.poisoned.load(Ordering::Relaxed) {
            Err(PoisonError::new(guard))
        } else {
            Ok(guard)
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Relaxed)
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        if self.poisoned.load(Ordering::Relaxed) {
            Err(PoisonError::new(data))
        } else {
            Ok(data)
        }
    }
}

pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
        self.lock.writer_locked.store(false, Ordering::Release);
    }
}
