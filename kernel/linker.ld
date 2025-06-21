ENTRY(_start)

SECTIONS {

    . = 0xffffffff80000000;
    . = ALIGN(0x1000);
    .text . :
    {
        *(.text .text.*)
    }

    . = ALIGN(0x1000);
    .rodata :
    {
      *(.rodata .rodata.*)
    }

    . = ALIGN(0x1000);
    .data :
    {
      *(.data .data.*)
    }

    . = ALIGN(0x1000);
    .bss :
    {
      *(COMMON)
      *(.bss .bss.*)
    }

    . = ALIGN(0x1000);
    .stack :
    {
      *(COMMON)
      *(.stack .stack.*)
    }
}