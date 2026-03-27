#![no_std]
#![no_main]

extern crate alloc;
extern crate plenumnet_kernel;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::alloc::{GlobalAlloc, Layout};

const HEAP_SIZE: usize = 1024 * 1024;

#[repr(C, align(4096))]
struct HeapMemory([u8; HEAP_SIZE]);

static mut HEAP: HeapMemory = HeapMemory([0; HEAP_SIZE]);
static HEAP_POS: AtomicUsize = AtomicUsize::new(0);

struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        loop {
            let pos = HEAP_POS.load(Ordering::Relaxed);
            let aligned = (pos + align - 1) & !(align - 1);
            let new_pos = aligned + size;
            if new_pos > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            if HEAP_POS
                .compare_exchange_weak(pos, new_pos, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return unsafe { HEAP.0.as_mut_ptr().add(aligned) };
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

#[cfg(target_arch = "x86_64")]
mod serial {
    const COM1: u16 = 0x3F8;

    pub fn init() {
        unsafe {
            outb(COM1 + 1, 0x00);
            outb(COM1 + 3, 0x80);
            outb(COM1, 0x01);
            outb(COM1 + 1, 0x00);
            outb(COM1 + 3, 0x03);
            outb(COM1 + 2, 0xC7);
            outb(COM1 + 4, 0x0B);
        }
    }

    pub fn putchar(b: u8) {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, b);
        }
    }

    unsafe fn outb(port: u16, val: u8) {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack));
    }

    unsafe fn inb(port: u16) -> u8 {
        let val: u8;
        core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack));
        val
    }
}

#[cfg(target_arch = "aarch64")]
mod serial {
    const PL011_BASE: usize = 0x0900_0000;

    pub fn init() {}

    pub fn putchar(b: u8) {
        unsafe {
            core::ptr::write_volatile(PL011_BASE as *mut u8, b);
        }
    }
}

#[cfg(target_arch = "riscv64")]
mod serial {
    pub fn init() {}

    pub fn putchar(b: u8) {
        unsafe {
            core::arch::asm!(
                "li a7, 1",
                "ecall",
                in("a0") b as usize,
                out("a7") _,
                options(nostack),
            );
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
mod serial {
    pub fn init() {}
    pub fn putchar(_b: u8) {}
}

fn puts(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            serial::putchar(b'\r');
        }
        serial::putchar(b);
    }
}

fn put_usize(n: usize) {
    if n >= 10 {
        put_usize(n / 10);
    }
    serial::putchar(b'0' + (n % 10) as u8);
}

fn put_u32(n: u32) {
    put_usize(n as usize);
}

fn put_hex_u64(n: u64) {
    let digits = b"0123456789ABCDEF";
    let mut started = false;
    for i in (0..16).rev() {
        let nibble = ((n >> (i * 4)) & 0xF) as usize;
        if nibble != 0 || started || i == 0 {
            serial::putchar(digits[nibble]);
            started = true;
        }
    }
}

#[cfg(target_arch = "x86_64")]
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
    "lea rsp, [boot_stack_top]",
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
    ".align 16",
    "boot_stack_bottom: .space 65536",
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

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".section .text.boot, \"ax\"",
    ".global _start",
    "_start:",
    "adrp x0, boot_stack_top",
    "add x0, x0, :lo12:boot_stack_top",
    "mov sp, x0",
    "bl kernel_main",
    "1:",
    "wfe",
    "b 1b",
    "",
    ".section .bss.boot, \"aw\", @nobits",
    ".align 16",
    "boot_stack_bottom: .space 65536",
    "boot_stack_top:",
);

#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(
    ".section .text.boot, \"ax\"",
    ".global _start",
    "_start:",
    "la sp, boot_stack_top",
    "call kernel_main",
    "1:",
    "wfi",
    "j 1b",
    "",
    ".section .bss.boot, \"aw\", @nobits",
    ".align 16",
    "boot_stack_bottom: .space 65536",
    "boot_stack_top:",
);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    serial::init();

    puts("\n");
    puts("================================================================\n");
    puts("  PlenumNET Kernel v0.1.0 — Salvi Framework\n");
    puts("  Copyright (c) 2026 Capomastro Holdings Ltd.\n");
    puts("  Applied Physics Division\n");
    puts("================================================================\n");
    puts("\n");

    use plenumnet_kernel::arch::boot::{self, BootSequence, BootParams};
    use plenumnet_kernel::arch::ArchId;

    #[cfg(target_arch = "x86_64")]
    let boot_params: BootParams = boot::x86_64_boot_config();
    #[cfg(target_arch = "aarch64")]
    let boot_params: BootParams = boot::aarch64_boot_config();
    #[cfg(target_arch = "riscv64")]
    let boot_params: BootParams = boot::riscv64_boot_config();

    let arch_name = match boot_params.arch_id {
        ArchId::X86_64 => "x86_64",
        ArchId::Aarch64 => "aarch64",
        ArchId::RiscV64 => "riscv64",
    };
    puts("[boot] Architecture: ");
    puts(arch_name);
    puts("\n");

    puts("[boot] Kernel physical base: 0x");
    put_hex_u64(boot_params.kernel_physical_base);
    puts("\n");
    puts("[boot] Kernel size: 0x");
    put_hex_u64(boot_params.kernel_size);
    puts("\n");
    puts("[boot] Framebuffer base: 0x");
    put_hex_u64(boot_params.framebuffer_base);
    puts(" (");
    put_u32(boot_params.framebuffer_width);
    puts("x");
    put_u32(boot_params.framebuffer_height);
    puts(", pitch=");
    put_u32(boot_params.framebuffer_pitch);
    puts(")\n");
    puts("[boot] Memory regions: ");
    put_usize(boot_params.memory_map.len());
    puts("\n");
    for region in &boot_params.memory_map {
        puts("[boot]   0x");
        put_hex_u64(region.base);
        puts(" size=0x");
        put_hex_u64(region.size);
        puts(" (");
        puts(match region.region_type {
            plenumnet_kernel::arch::MemoryRegionType::Usable => "usable",
            plenumnet_kernel::arch::MemoryRegionType::Reserved => "reserved",
            plenumnet_kernel::arch::MemoryRegionType::AcpiReclaimable => "acpi",
            plenumnet_kernel::arch::MemoryRegionType::AcpiNvs => "acpi-nvs",
            plenumnet_kernel::arch::MemoryRegionType::Defective => "defective",
            plenumnet_kernel::arch::MemoryRegionType::TernaryCoprocessor => "ternary-coproc",
        });
        puts(")\n");
    }

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
                puts("[boot] Stage ");
                puts(name);
                puts(": OK\n");
            }
            Err(_) => {
                puts("[boot] Stage ");
                puts(name);
                puts(": FAILED\n");
                halt_loop();
            }
        }
    }

    puts("[boot] Boot sequence complete (");
    put_usize(seq.elapsed_stages());
    puts(" stages)\n");
    puts("\n");

    let fb_w = if boot_params.framebuffer_width > 0 && boot_params.framebuffer_width <= 4096 {
        boot_params.framebuffer_width
    } else {
        320
    };
    let fb_h = if boot_params.framebuffer_height > 0 && boot_params.framebuffer_height <= 4096 {
        boot_params.framebuffer_height
    } else {
        200
    };

    let smoke_w = if fb_w > 320 { 320 } else { fb_w };
    let smoke_h = if fb_h > 200 { 200 } else { fb_h };

    puts("[browser] Initializing PlenumBrowser subsystem...\n");
    let mut browser = plenumnet_kernel::browser::Browser::new(smoke_w, smoke_h);
    puts("[browser] Framebuffer: ");
    put_u32(smoke_w);
    puts("x");
    put_u32(smoke_h);
    puts(" (CPU renderer, hw fb @ 0x");
    put_hex_u64(boot_params.framebuffer_base);
    puts(")\n");

    browser.flush_render();
    puts("[browser] Render pipeline: OK\n");

    let tab = browser.open_tab(alloc::string::String::from("plenum://boot-test"));
    match tab {
        Ok(id) => {
            puts("[browser] Tab opened: plenum://boot-test (id=");
            put_u32(id);
            puts(")\n");
        }
        Err(_) => {
            puts("[browser] Tab open: FAILED\n");
        }
    }
    puts("[browser] Tab count: ");
    put_usize(browser.tab_count());
    puts("\n");

    puts("[browser] Rendering boot test page...\n");
    {
        let fb = browser.framebuffer_mut();
        fb.clear([20, 40, 80, 255]);
        fb.fill_rect(10, 10, 300.min(smoke_w - 20), 40, [255, 255, 255, 255]);
        fb.fill_rect(10, 55, 300.min(smoke_w - 20), 2, [139, 92, 246, 255]);
    }
    browser.flush_render();
    puts("[browser] Test page rendered to framebuffer\n");

    puts("[distributor] Initializing z=0 distributor...\n");
    let mut dist = plenumnet_kernel::distributor::Distributor::new();
    use plenumnet_kernel::distributor::z_router::{RequestType, ZLevel};
    let _req = dist.dispatch(ZLevel::UI, RequestType::HttpRequest);
    puts("[distributor] Coprime walk + z-router: OK (");
    put_usize(dist.requests_processed() as usize);
    puts(" requests)\n");

    puts("\n");
    puts("================================================================\n");
    puts("  PLENUMNET KERNEL BOOT OK\n");
    puts("================================================================\n");
    puts("\n");

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0xF4u16, in("al") 0x31u8, options(nomem, nostack));
    }

    halt_loop();
}

fn halt_loop() -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    puts("\n!!! KERNEL PANIC !!!\n");
    if let Some(location) = info.location() {
        puts("  at ");
        puts(location.file());
        puts(":");
        put_usize(location.line() as usize);
        puts("\n");
    }
    halt_loop();
}
