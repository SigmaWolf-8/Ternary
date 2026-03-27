use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

const INCOMPLETE: u8 = 0;
const RUNNING: u8 = 1;
const COMPLETE: u8 = 2;

pub struct Once {
    state: AtomicU8,
}

impl Once {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(INCOMPLETE),
        }
    }

    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            return;
        }
        match self
            .state
            .compare_exchange(INCOMPLETE, RUNNING, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => {
                f();
                self.state.store(COMPLETE, Ordering::Release);
            }
            Err(_) => {
                while self.state.load(Ordering::Acquire) != COMPLETE {
                    core::hint::spin_loop();
                }
            }
        }
    }

    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}

pub struct OnceLock<T> {
    once: Once,
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send + Sync> Send for OnceLock<T> {}
unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            value: UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            unsafe { (*self.value.get()).as_ref() }
        } else {
            None
        }
    }

    pub fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> &T {
        self.once.call_once(|| {
            unsafe {
                *self.value.get() = Some(f());
            }
        });
        unsafe { (*self.value.get()).as_ref().unwrap() }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        let mut value = Some(value);
        self.once.call_once(|| {
            unsafe {
                *self.value.get() = value.take();
            }
        });
        match value {
            None => Ok(()),
            Some(v) => Err(v),
        }
    }

    pub fn into_inner(self) -> Option<T> {
        self.value.into_inner()
    }
}
