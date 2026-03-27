#![allow(clippy::new_without_default)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::time::Duration;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
static CURRENT_TASK_ID: AtomicU64 = AtomicU64::new(0);

pub fn set_current_task_id(id: u64) {
    CURRENT_TASK_ID.store(id, Ordering::Release);
}

fn current_task_id() -> u64 {
    CURRENT_TASK_ID.load(Ordering::Acquire)
}

pub struct Builder {
    name: Option<String>,
    stack_size: Option<usize>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            name: None,
            stack_size: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }

    pub fn spawn<F, T>(self, f: F) -> crate::io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        Ok(spawn_inner(f, self.name))
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    spawn_inner(f, None)
}

fn spawn_inner<F, T>(f: F, name: Option<String>) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    let result_slot: Arc<TaskResult<T>> = Arc::new(TaskResult {
        value: UnsafeCell::new(None),
        completed: AtomicBool::new(false),
    });

    let result_clone = result_slot.clone();

    if let Some(spawner) = unsafe { TASK_SPAWNER } {
        let boxed_fn: Box<dyn FnOnce() + Send + 'static> = Box::new({
            let result_clone2 = result_clone.clone();
            move || {
                let prev_id = current_task_id();
                set_current_task_id(task_id);
                let result = f();
                unsafe {
                    *result_clone2.value.get() = Some(result);
                }
                result_clone2.completed.store(true, Ordering::Release);
                set_current_task_id(prev_id);
            }
        });
        spawner(task_id, boxed_fn);
    } else {
        let prev_id = current_task_id();
        set_current_task_id(task_id);
        let result = f();
        unsafe {
            *result_clone.value.get() = Some(result);
        }
        result_clone.completed.store(true, Ordering::Release);
        set_current_task_id(prev_id);
    }

    JoinHandle {
        task_id,
        result: result_slot,
        name,
    }
}

type TaskSpawnerFn = fn(u64, Box<dyn FnOnce() + Send + 'static>);
static mut TASK_SPAWNER: Option<TaskSpawnerFn> = None;

pub fn register_task_spawner(spawner: TaskSpawnerFn) {
    unsafe {
        TASK_SPAWNER = Some(spawner);
    }
}

struct TaskResult<T> {
    value: UnsafeCell<Option<T>>,
    completed: AtomicBool,
}

unsafe impl<T: Send> Send for TaskResult<T> {}
unsafe impl<T: Send> Sync for TaskResult<T> {}

pub struct JoinHandle<T> {
    task_id: u64,
    result: Arc<TaskResult<T>>,
    name: Option<String>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, Box<dyn core::any::Any + Send + 'static>> {
        while !self.result.completed.load(Ordering::Acquire) {
            core::hint::spin_loop();
        }
        let value = unsafe { (*self.result.value.get()).take() };
        match value {
            Some(v) => Ok(v),
            None => Err(Box::new("task already joined")),
        }
    }

    pub fn thread(&self) -> Thread {
        Thread {
            id: ThreadId(self.task_id),
            name: self.name.clone(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.result.completed.load(Ordering::Relaxed)
    }
}

pub fn current() -> Thread {
    Thread {
        id: ThreadId(current_task_id()),
        name: Some(String::from("main")),
    }
}

pub fn sleep(dur: Duration) {
    let nanos = dur.as_nanos();
    let iterations = (nanos / 10) as u64;
    for _ in 0..iterations.min(100_000) {
        core::hint::spin_loop();
    }
}

pub fn yield_now() {
    core::hint::spin_loop();
}

pub fn park() {
    core::hint::spin_loop();
}

pub fn park_timeout(dur: Duration) {
    sleep(dur);
}

pub fn panicking() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Thread {
    id: ThreadId,
    name: Option<String>,
}

impl Thread {
    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn unpark(&self) {}
}

static TLS_LOCK: AtomicBool = AtomicBool::new(false);

struct TlsStorage {
    entries: UnsafeCell<Option<BTreeMap<(u64, usize), *mut u8>>>,
}

unsafe impl Send for TlsStorage {}
unsafe impl Sync for TlsStorage {}

static TLS_STORE: TlsStorage = TlsStorage {
    entries: UnsafeCell::new(None),
};

fn tls_lock() {
    while TLS_LOCK
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

fn tls_unlock() {
    TLS_LOCK.store(false, Ordering::Release);
}

pub struct LocalKey<T: 'static> {
    init: fn() -> T,
    _marker: core::marker::PhantomData<fn() -> T>,
}

unsafe impl<T: 'static> Sync for LocalKey<T> {}
unsafe impl<T: 'static> Send for LocalKey<T> {}

impl<T: 'static> LocalKey<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            init,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let task_id = current_task_id();
        let key_addr = self as *const _ as usize;

        tls_lock();
        unsafe {
            let store = &mut *TLS_STORE.entries.get();
            if store.is_none() {
                *store = Some(BTreeMap::new());
            }
            let map = store.as_mut().unwrap_unchecked();
            let entry = map.entry((task_id, key_addr));
            let ptr = match entry {
                alloc::collections::btree_map::Entry::Occupied(e) => *e.get(),
                alloc::collections::btree_map::Entry::Vacant(e) => {
                    let val = Box::new((self.init)());
                    let raw = Box::into_raw(val) as *mut u8;
                    e.insert(raw);
                    raw
                }
            };
            tls_unlock();
            let val_ref = &*(ptr as *const T);
            f(val_ref)
        }
    }

    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        Ok(self.with(f))
    }
}

#[derive(Debug)]
pub struct AccessError;

impl core::fmt::Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "already destroyed")
    }
}

pub fn available_parallelism() -> crate::io::Result<core::num::NonZeroUsize> {
    Ok(unsafe { core::num::NonZeroUsize::new_unchecked(1) })
}
