    global _start
    extern start


    MULTIBOOT_MAGIC        equ 0xE85250D6
    MULTIBOOT_ARCHITECTURE equ 0
    MULTIBOOT_LENGTH       equ (multiboot_end - multiboot)


    section .multiboot
    align 8
multiboot:
    dd MULTIBOOT_MAGIC
    dd MULTIBOOT_ARCHITECTURE
    dd MULTIBOOT_LENGTH
    dd -(MULTIBOOT_MAGIC + MULTIBOOT_ARCHITECTURE + MULTIBOOT_LENGTH)

    align 8
    dw 0
    dw 0
    dd 8
multiboot_end:


    section .text

_start:
    mov esp, stack_end
    call load_gdt
    call start
.stop:
    hlt
    jmp .stop

load_gdt:
    lgdt [gdt.pointer]
    jmp 0x08:.set_registers
.set_registers:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret


    section .bss
stack:
    resb 0x4000
stack_end:


    section .rodata
gdt:
    dq 0
.code:
    dw 0xFFFF
    dw 0
    db 0
    db 0x9A
    db 11001111b
    db 0
.data:
    dw 0xFFFF
    dw 0
    db 0
    db 0x92
    db 11001111b
    db 0
.pointer:
    dw $ - gdt - 1
    dd gdt
