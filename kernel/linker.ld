ENTRY(_start);

SECTIONS
{
    . = 0x100000;

    .text :
    {
        *(.text._start)
        *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }

    .data :
    {
        *(.data*)
    }

    .bss :
    {
        *(COMMON)
        *(.bss*)
    }
}
