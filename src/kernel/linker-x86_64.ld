ENTRY(_start)

SECTIONS
{
    . = 1M;

    .multiboot2 ALIGN(8) :
    {
        KEEP(*(.multiboot2))
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

    .bss ALIGN(4K) :
    {
        __bss_start = .;
        KEEP(*(.bss.boot))
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    .got ALIGN(4K) :
    {
        *(.got .got.*)
    }

    /DISCARD/ :
    {
        *(.comment)
        *(.note.*)
        *(.eh_frame*)
    }
}
