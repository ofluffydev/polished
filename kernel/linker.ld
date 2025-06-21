ENTRY(naked_start)

SECTIONS {

    . = 0xffffffff80000000;
    . = ALIGN(0x1000);
    .text : ALIGN(4K)
    {
        *(.text.init .text .text.* .text*)
    } :text

    . = ALIGN(0x1000);
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.* .rodata*)
        *(.srodata .srodata.* .srodata*)
    } :rodata

    . = ALIGN(0x1000);
    .data : ALIGN(4K)
    {
        *(.data .data.* .data*)
        *(.sdata .sdata.* .sdata*)
    } :data

    . = ALIGN(0x1000);
    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.* .bss*)
        *(.sbss .sbss.* .sbss*)
        // cant trust uefi
        . = ALIGN(4096);
        PROVIDE(STACK_BOTTOM = .);
        . += 65536; /* Increased stack size to 64 KiB */
        PROVIDE(STACK_TOP = .);
    } :bss
}