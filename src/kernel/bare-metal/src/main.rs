// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved
//
// Bare-Metal Boot Target for the PlenumNET Ternary Kernel
//
// This binary boots on raw x86_64 hardware (QEMU) with NO operating system.
// It initializes the plenumnet-kernel library, exercises every subsystem,
// and runs a self-test suite — proving the kernel is a real bootable system.

#![no_std]
#![no_main]

extern crate alloc;

mod serial;
mod selftest;

use core::panic::PanicInfo;

// ─────────────────────────────────────────────────────────────────────
// GLOBAL ALLOCATOR — required by plenumnet-kernel (uses alloc crate)
// Bump allocator backed by the heap region from linker.ld
// ─────────────────────────────────────────────────────────────────────

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

struct BumpAllocator {
    next: AtomicUsize,
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap_start = unsafe { &__heap_start as *const u8 as usize };
        let heap_end = unsafe { &__heap_end as *const u8 as usize };

        loop {
            let current = self.next.load(Ordering::Relaxed);
            let actual = if current == 0 { heap_start } else { current };
            let aligned = (actual + layout.align() - 1) & !(layout.align() - 1);
            let new_next = aligned + layout.size();

            if new_next > heap_end {
                return core::ptr::null_mut();
            }

            if self.next.compare_exchange(current, new_next, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                return aligned as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free — acceptable for boot self-test
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    next: AtomicUsize::new(0),
};

extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
    static __stack_top: u8;
    static __heap_start: u8;
    static __heap_end: u8;
}

// ─────────────────────────────────────────────────────────────────────
// BOOT ENTRY POINT
// ─────────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
#[link_section = ".text.boot"]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "lea rsp, [rip + __stack_top]",
            options(nostack, nomem)
        );
    }

    zero_bss();
    serial::init();

    serial::print_line("================================================================");
    serial::print_line("  PlenumNET Ternary Kernel — Bare-Metal Validation");
    serial::print_line("  Salvi Framework v0.1.0");
    serial::print_line("  Capomastro Holdings Ltd. — Applied Physics Division");
    serial::print_line("  Target: x86_64-unknown-none (QEMU)");
    serial::print_line("================================================================");
    serial::print_line("");

    serial::print_str("[BOOT] Kernel version: ");
    serial::print_line(plenumnet_kernel::KERNEL_VERSION);
    serial::print_str("[BOOT] Copyright: ");
    serial::print_line(plenumnet_kernel::COPYRIGHT);

    serial::print_line("[BOOT] Initializing kernel...");
    let info = plenumnet_kernel::init();
    serial::print_str("[BOOT] Architecture: ");
    serial::print_line(match info.architecture {
        plenumnet_kernel::Architecture::X86_64 => "x86_64",
        plenumnet_kernel::Architecture::Aarch64 => "aarch64",
        plenumnet_kernel::Architecture::Riscv64 => "riscv64",
        plenumnet_kernel::Architecture::Fpga => "FPGA",
        plenumnet_kernel::Architecture::Asic => "ASIC",
        plenumnet_kernel::Architecture::Unknown => "unknown",
    });
    serial::print_str("[BOOT] Timing source: ");
    serial::print_line(match info.timing_source {
        plenumnet_kernel::TimingSource::Tsc => "TSC",
        plenumnet_kernel::TimingSource::Hpet => "HPET",
        plenumnet_kernel::TimingSource::Ptp => "PTP",
        plenumnet_kernel::TimingSource::OpticalAtomic => "Optical Atomic",
        plenumnet_kernel::TimingSource::SystemClock => "System Clock",
        plenumnet_kernel::TimingSource::Unknown => "unknown",
    });
    serial::print_line("[BOOT] Kernel initialized successfully.");
    serial::print_line("");

    serial::print_line("[TEST] Running bare-metal self-test suite...");
    serial::print_line("    Tests exercise the ACTUAL plenumnet-kernel library code.");
    serial::print_line("    No OS, no libc, no allocator runtime — just raw hardware.");
    serial::print_line("----------------------------------------------------------------");
    let results = selftest::run_all();

    serial::print_line("----------------------------------------------------------------");
    serial::print_str("[RESULT] ");
    serial::print_u64(results.passed as u64);
    serial::print_str(" passed, ");
    serial::print_u64(results.failed as u64);
    serial::print_line(" failed");

    if results.failed == 0 {
        serial::print_line("");
        serial::print_line("[PASS] BARE-METAL VALIDATION PASSED");
        serial::print_line("       The PlenumNET kernel boots and runs on raw hardware.");
        exit_qemu(QemuExitCode::Success);
    } else {
        serial::print_line("");
        serial::print_line("[FAIL] BARE-METAL VALIDATION FAILED");
        exit_qemu(QemuExitCode::Failure);
    }

    halt_loop()
}

fn zero_bss() {
    unsafe {
        let start = &__bss_start as *const u8 as *mut u8;
        let end = &__bss_end as *const u8 as *mut u8;
        let len = end as usize - start as usize;
        core::ptr::write_bytes(start, 0, len);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(code: QemuExitCode) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xf4u16,
            in("eax") code as u32,
            options(nostack, nomem)
        );
    }
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nostack, nomem)); }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::print_line("");
    serial::print_line("!!! KERNEL PANIC !!!");
    if let Some(location) = info.location() {
        serial::print_str("  at ");
        serial::print_str(location.file());
        serial::print_str(":");
        serial::print_u64(location.line() as u64);
        serial::print_line("");
    }
    if let Some(msg) = info.message().as_str() {
        serial::print_str("  ");
        serial::print_line(msg);
    }
    exit_qemu(QemuExitCode::Failure);
    halt_loop()
}
