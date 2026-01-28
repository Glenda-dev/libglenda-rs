ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(0x1000); /* 强制 4KB 对齐，确保下一个段进入新页面 */
    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(0x1000); /* 强制 4KB 对齐，确保下一个段进入新页面 */

    .data : {
        . = ALIGN(4);
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.*)
        *(.data .data.*)
    }
    . = ALIGN(0x1000); /* 强制 4KB 对齐，确保下一个段进入新页面 */

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}
