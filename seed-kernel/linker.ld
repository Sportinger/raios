OUTPUT("seed-kernel.elf")
ENTRY(_start)

SECTIONS
{
    . = 1M;

    .text ALIGN(4K) :
    {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata ALIGN(4K) :
    {
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) :
    {
        *(.data .data.*)
    }

    .bss ALIGN(4K) (NOLOAD) :
    {
        __bss_start = .;
        *(.bss .bss.* COMMON)
        __bss_end = .;
    }

    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
