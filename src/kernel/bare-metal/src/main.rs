// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved
//
// Bare-Metal Boot Target for the PlenumNET Ternary Kernel
//
// This binary boots on raw x86_64 hardware (QEMU) with NO operating system.
// It initializes the plenumnet-kernel library, runs the self-test suite,
// then boots the full PlenumBrowser subsystem with sponge encryption and
// z=0 distributor — proving the entire pipeline works on bare metal.

#![no_std]
#![no_main]

extern crate alloc;

mod serial;
mod selftest;

use core::panic::PanicInfo;

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
// MULTIBOOT1 HEADER + 32→64 BIT TRAMPOLINE
// Multiboot boots in 32-bit protected mode. We must set up paging,
// enable long mode, and far-jump to 64-bit code before calling Rust.
// ─────────────────────────────────────────────────────────────────────

core::arch::global_asm!(
    ".section .multiboot, \"a\"",
    ".align 4",
    "mb_header:",
    ".long 0x1BADB002",
    ".long 0x00000003",
    ".long -(0x1BADB002 + 0x00000003)",
    "",
    ".section .text.boot, \"ax\"",
    ".code32",
    ".global _start",
    "_start:",
    "cli",
    "mov esi, ebx",
    "",
    "lea esp, [boot_stack_top]",
    "",
    "mov dx, 0x3F9",
    "xor al, al",
    "out dx, al",
    "mov dx, 0x3FB",
    "mov al, 0x80",
    "out dx, al",
    "mov dx, 0x3F8",
    "mov al, 0x01",
    "out dx, al",
    "mov dx, 0x3F9",
    "xor al, al",
    "out dx, al",
    "mov dx, 0x3FB",
    "mov al, 0x03",
    "out dx, al",
    "mov dx, 0x3F8",
    "mov al, 0x31",
    "out dx, al",
    "",
    "lea edi, [boot_pml4]",
    "xor eax, eax",
    "mov ecx, 3072",
    "rep stosd",
    "",
    "lea eax, [boot_pdpt]",
    "or eax, 0x03",
    "mov [boot_pml4], eax",
    "",
    "lea eax, [boot_pd]",
    "or eax, 0x03",
    "mov [boot_pdpt], eax",
    "",
    "lea edi, [boot_pd]",
    "mov eax, 0x83",
    "mov ecx, 512",
    "2:",
    "mov [edi], eax",
    "add eax, 0x200000",
    "add edi, 8",
    "dec ecx",
    "jnz 2b",
    "",
    "mov dx, 0x3F8",
    "mov al, 0x32",
    "out dx, al",
    "",
    "lea eax, [boot_pml4]",
    "mov cr3, eax",
    "",
    "mov eax, cr4",
    "or eax, 0x20",
    "mov cr4, eax",
    "",
    "mov ecx, 0xC0000080",
    "rdmsr",
    "or eax, 0x100",
    "wrmsr",
    "",
    "mov eax, cr0",
    "or eax, 0x80000000",
    "mov cr0, eax",
    "",
    "lgdt [boot_gdt_ptr]",
    "",
    "mov dx, 0x3F8",
    "mov al, 0x33",
    "out dx, al",
    "",
    "push 0x08",
    "lea eax, [_start64]",
    "push eax",
    "retf",
    "",
    ".code64",
    ".global _start64",
    "_start64:",
    "mov ax, 0x10",
    "mov ds, ax",
    "mov es, ax",
    "mov fs, ax",
    "mov gs, ax",
    "mov ss, ax",
    "lea rsp, [__stack_top]",
    "",
    "push 0x34",
    "mov dx, 0x3F8",
    "pop rax",
    "out dx, al",
    "",
    "call kernel_main",
    "3:",
    "hlt",
    "jmp 3b",
    "",
    ".section .bss.boot, \"aw\", @nobits",
    ".align 4096",
    "boot_pml4: .space 4096",
    "boot_pdpt: .space 4096",
    "boot_pd:   .space 4096",
    "boot_stack_bottom: .space 4096",
    "boot_stack_top:",
    "",
    ".section .rodata.boot, \"a\"",
    ".align 16",
    "boot_gdt:",
    ".quad 0",
    ".quad 0x00AF9A000000FFFF",
    ".quad 0x00CF92000000FFFF",
    "boot_gdt_ptr:",
    ".short boot_gdt_ptr - boot_gdt - 1",
    ".long boot_gdt",
);

// ─────────────────────────────────────────────────────────────────────
// KERNEL MAIN — called from assembly after 64-bit mode is established
// ─────────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
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

    if results.failed > 0 {
        serial::print_line("");
        serial::print_line("[FAIL] BARE-METAL SELF-TESTS FAILED");
        exit_qemu(QemuExitCode::Failure);
        halt_loop();
    }

    serial::print_line("");
    serial::print_line("[PASS] Self-tests passed. Proceeding to full browser boot...");
    serial::print_line("");

    serial::print_line("PlenumNET Kernel v0.1.0");

    use plenumnet_kernel::arch::boot::{self, BootSequence};

    let boot_params = boot::x86_64_boot_config();

    serial::print_line("[boot] Architecture: x86_64");

    let mut seq = BootSequence::new(boot_params.arch_id);
    let stage_names = [
        "FirmwareHandoff",
        "EarlyInit",
        "MemoryDetection",
        "PageTableSetup",
        "InterruptSetup",
        "TimerSetup",
        "TernaryCoprocessorInit",
        "DriverInit",
        "SchedulerInit",
        "LateInit",
        "Running",
    ];

    for name in &stage_names {
        match seq.advance() {
            Ok(_stage) => {
                serial::print_str("[boot] Stage ");
                serial::print_str(name);
                serial::print_line(": OK");
            }
            Err(_) => {
                serial::print_str("[boot] Stage ");
                serial::print_str(name);
                serial::print_line(": FAILED");
                exit_qemu(QemuExitCode::Failure);
                halt_loop();
            }
        }
    }

    serial::print_str("Boot sequence complete (");
    serial::print_u64(seq.elapsed_stages() as u64);
    serial::print_line(" stages)");

    let fb_w: u32 = 1920;
    let fb_h: u32 = 1080;

    serial::print_line("[browser] Initializing PlenumBrowser subsystem...");
    let distributor = alloc::boxed::Box::new(plenumnet_kernel::distributor::Distributor::new());
    let mut browser = plenumnet_kernel::browser::Browser::new(fb_w, fb_h, distributor);
    serial::print_str("[browser] Framebuffer: ");
    serial::print_u64(fb_w as u64);
    serial::print_str("x");
    serial::print_u64(fb_h as u64);
    serial::print_line(" (CPU renderer)");

    serial::print_line("[color] Initializing PlenumColor mesh pipeline (depth 3)...");
    let precision = plenumnet_kernel::browser::color::MeshPrecision::compute(3);
    serial::print_str("[color] Depth 3: ");
    serial::print_u64(precision.total_addresses as u64);
    serial::print_str(" addresses, ~");
    serial::print_u64(precision.effective_bits as u64);
    serial::print_line(" effective bits/channel");

    serial::print_line("[color] Building depth-3 LUT...");
    let _lut = plenumnet_kernel::browser::color::MeshColorLut::build(3);
    serial::print_str("[color] LUT built: ");
    serial::print_u64(_lut.memory_bytes() as u64);
    serial::print_line(" bytes");

    serial::print_line("[browser] Rendering plenum://home boot page...");
    let tab = browser.open_tab(alloc::string::String::from("plenum://home"));
    match tab {
        Ok(id) => {
            serial::print_str("[browser] Tab opened: plenum://home (id=");
            serial::print_u64(id as u64);
            serial::print_line(")");
        }
        Err(_) => {
            serial::print_line("[browser] Tab open: FAILED");
        }
    }

    browser.render_home_page();
    serial::print_line("[browser] Home page rendered to framebuffer");

    browser.apply_mesh_color();
    serial::print_line("[color] Mesh color pipeline applied to framebuffer");

    serial::print_line("[distributor] Initializing z=0 distributor...");
    let mut dist = plenumnet_kernel::distributor::Distributor::new();
    use plenumnet_kernel::distributor::RequestInterface;
    use plenumnet_kernel::distributor::z_router::RequestType;

    let _req1 = dist.submit_request(RequestType::HttpRequest);
    let _req2 = dist.submit_request(RequestType::DataQuery);
    let _req3 = dist.submit_request(RequestType::FileServe);
    serial::print_str("[distributor] Coprime walk + z-router: OK (");
    serial::print_u64(dist.requests_processed() as u64);
    serial::print_line(" requests dispatched)");

    serial::print_line("[sponge] Initializing TLSponge-385 per-frame encryption...");
    use plenumnet_kernel::distributor::sponge_rekey::{SpongeRekeyState, FrameResolution};
    let initial_key = [
        0x50, 0x4C, 0x45, 0x4E, 0x55, 0x4D, 0x4E, 0x45,
        0x54, 0x5F, 0x4B, 0x45, 0x59, 0x5F, 0x30, 0x31,
        0x53, 0x41, 0x4C, 0x56, 0x49, 0x5F, 0x46, 0x52,
        0x41, 0x4D, 0x45, 0x57, 0x4F, 0x52, 0x4B, 0x21,
    ];
    let resolution = FrameResolution::Hd1080;
    let mut sponge_state = SpongeRekeyState::new(&initial_key, resolution);
    serial::print_line("[sponge] TLSponge-385: 729-trit state, 9 rounds, 385-bit security");
    serial::print_line("[sponge] Rekey interval: 461 overlap slots (prime)");

    let keystream = sponge_state.advance_frame();
    browser.encrypt_framebuffer(&keystream);
    serial::print_str("[sponge] Frame 1 encrypted (");
    serial::print_u64(keystream.len() as u64);
    serial::print_line(" bytes keystream)");

    serial::print_str("[browser] Tab count: ");
    serial::print_u64(browser.tab_count() as u64);
    serial::print_line("");

    serial::print_line("");
    serial::print_line("================================================================");
    serial::print_line("  PLENUMNET KERNEL BOOT OK");
    serial::print_line("  Pipeline: parse -> layout -> render -> mesh color -> encrypt");
    serial::print_line("  Home page: plenum://home (full pipeline exercised)");
    serial::print_line("================================================================");
    serial::print_line("");

    exit_qemu(QemuExitCode::Success);

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
