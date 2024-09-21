ENTRY(__start)

OUTPUT_FORMAT(binary)

SECTIONS {
    . = 0x100000;

    .multiboot_header : {
        __multiboot_start = .;
        // magic (required)
        LONG(0x1BADB002);
        // flags (required)
        LONG(0x00010003);
        // checksum (required)
        LONG(-(0x1BADB002 + 0x00010003));
        // header_addr (present if flags[16] is set)
        LONG(__multiboot_start);
        // load_addr (present if flags[16] is set)
        LONG(__multiboot_start);
        // load_end_addr (present if flags[16] is set)
        LONG(__data_end);
        // bss_end_addr (present if flags[16] is set)
        LONG(__bss_end);
        // entry_addr (present if flags[16] is set)
        LONG(__start);
        __multiboot_end = .;
    }

    .text : ALIGN(CONSTANT(MAXPAGESIZE)) {
        __text_start = .;
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : ALIGN(CONSTANT(MAXPAGESIZE)) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : ALIGN(CONSTANT(MAXPAGESIZE)) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss (NOLOAD) : ALIGN(CONSTANT(MAXPAGESIZE)) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame .eh_frame*)
    }
}
