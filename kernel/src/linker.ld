ENTRY(_start)

SECTIONS {
    . = 1M;
    _kernel_start = .;

    .text :
    {
        KEEP(src/boot.o(.text))
        *(.text)
        *(.text.*)
    }

    .rodata :
    {
        *(.rodata)
        *(.rodata.*)
    }

    .data :
    {
        *(.data)
        *(.data.*)
    }

    .bss :
    {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    }

    _kernel_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.debug_*)
        *(.note.*)
        *(.comment)
    }
}
