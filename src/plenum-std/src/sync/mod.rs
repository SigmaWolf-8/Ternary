pub use alloc::sync::{Arc, Weak};

mod mutex_impl;
mod rwlock_impl;
mod once_impl;
mod condvar_impl;
mod barrier_impl;
pub mod mpsc;

pub use self::mutex_impl::{Mutex, MutexGuard, PoisonError, LockResult, TryLockError};
pub use self::rwlock_impl::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use self::once_impl::{Once, OnceLock};
pub use self::condvar_impl::Condvar;
pub use self::barrier_impl::{Barrier, BarrierWaitResult};

pub mod atomic {
    pub use core::sync::atomic::*;
}
