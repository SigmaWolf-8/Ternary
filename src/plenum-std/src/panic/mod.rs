pub use core::panic::*;

pub fn catch_unwind<F: FnOnce() -> R + core::panic::UnwindSafe, R>(f: F) -> Result<R, alloc::boxed::Box<dyn core::any::Any + Send>> {
    Ok(f())
}

pub fn resume_unwind(_payload: alloc::boxed::Box<dyn core::any::Any + Send>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn set_hook(_hook: alloc::boxed::Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>) {}

pub fn take_hook() -> alloc::boxed::Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static> {
    alloc::boxed::Box::new(|_| {})
}

pub use core::panic::PanicInfo as PanicHookInfo;
