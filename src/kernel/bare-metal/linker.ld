/* PlenumNET Ternary Kernel — Bare-Metal Linker Script
 * Target: x86_64, loaded by QEMU at 0x100000 (1 MiB)
 * Matches kernel_physical_base from arch::boot::x86_64_boot_config()
 *
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd.
 * Applied Physics Division — Salvi Framework
 */

ENTRY(_start)

SECTIONS
{
    . = 0x100000;

    .multiboot ALIGN(4) :
    {
        KEEP(*(.multiboot))
    }

    .text.boot ALIGN(4K) :
    {
        KEEP(*(.text.boot))
    }

    .text ALIGN(4K) :
    {
        *(.text .text.*)
    }

    .rodata ALIGN(4K) :
    {
        KEEP(*(.rodata.boot))
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) :
    {
        *(.data .data.*)
    }

    .bss.boot ALIGN(4K) (NOLOAD) :
    {
        KEEP(*(.bss.boot))
    }

    .bss ALIGN(4K) :
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4K);
    __stack_bottom = .;
    . += 64K;
    __stack_top = .;

    . = ALIGN(4K);
    __heap_start = .;
    . += 32M;
    __heap_end = .;

    __kernel_end = .;

    .note : { *(.note.Xen) }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.comment)
        *(.note.GNU-stack)
    }
}
