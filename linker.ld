ENTRY(_start)

SECTIONS {
    . = 1M;

    .multiboot : {
        KEEP(*(.multiboot))
    }
}
