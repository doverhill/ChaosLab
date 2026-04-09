; AP trampoline with binary search for fault location.
; The BSP reads the stage from [0x8008] after timeout.
; To find the fault, uncomment ONE "hlt_loop" at a time. If the AP halts
; (BSP reads the stage and AP doesn't crash), the fault is AFTER that stage.
; If the AP still crashes, the fault is BEFORE that stage.

ORG 0x8000
SECTION .text

trampoline:
    jmp short startup_ap
    times 8 - ($ - trampoline) nop
    .ready:       dq 0       ; offset 0x08 — debug stage / ready flag
    .page_table:  dq 0       ; offset 0x10: CR3 value
    .stack_top:   dq 0       ; offset 0x18: per-AP stack pointer
    .entry_point: dq 0       ; offset 0x20: Rust entry function address
    ; kernel GDT descriptor for lgdt (10 bytes: u16 limit + u64 base)
    .kernel_gdt:  dw 0       ; offset 0x28
                  dq 0       ; offset 0x2A
    times (0x38 - ($ - trampoline)) db 0  ; pad to 0x38
    ; kernel IDT descriptor for lidt (10 bytes: u16 limit + u64 base)
    .kernel_idt:  dw 0       ; offset 0x38
                  dq 0       ; offset 0x3A


BITS 16
startup_ap:
    cli
    cld

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov byte [0x8008], 1

    lgdt [gdtr32]
    mov byte [0x8008], 2

    mov eax, cr0
    or eax, 1
    mov cr0, eax
    mov byte [0x8008], 3

    jmp dword 0x18:protected_mode   ; 0x18 = 32-bit code

BITS 32
protected_mode:
    mov byte [0x8008], 4

    mov ax, 0x20                     ; 0x20 = 32-bit data
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov byte [0x8008], 5

    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax
    mov byte [0x8008], 6

    mov eax, [trampoline.page_table]
    mov cr3, eax
    mov byte [0x8008], 7

    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr
    mov byte [0x8008], 8

    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax
    mov byte [0x8008], 9

    jmp 0x08:long_mode               ; 0x08 = 64-bit code (matches kernel)

BITS 64
long_mode:
    mov byte [0x8008], 10

    ; Temporarily load data segments from our trampoline GDT (0x10)
    ; so we have a valid SS for stack operations while setting up.
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; load per-AP stack
    mov rsp, [trampoline.stack_top]

    ; enable SSE
    mov rax, cr0
    and ax, 0xFFFB      ; clear EM
    or ax, 0x0002       ; set MP
    mov cr0, rax
    mov rax, cr4
    or ax, (1 << 9) | (1 << 10)  ; OSFXSR + OSXMMEXCPT
    mov cr4, rax

    ; Load the kernel's GDT and IDT (patched by BSP).
    ; CS=0x08 is the same selector in both our trampoline GDT and the kernel
    ; GDT, so no CS reload is needed.
    ; Load data segments FIRST (using trampoline GDT's data at 0x10).
    ; Both trampoline and kernel GDTs have a valid data segment at 0x10.
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; serial 'S' = segments loaded
    mov dx, 0x3F8
    mov al, 0x53
    out dx, al

    ; Load kernel GDT — CS=0x08 and data=0x10 are identical in both GDTs
    lgdt [trampoline.kernel_gdt]

    ; serial 'G' = kernel GDT loaded
    mov al, 0x47
    out dx, al

    ; serial 'C' = CS reloaded
    mov dx, 0x3F8
    mov al, 0x43
    out dx, al

    ; Skip TSS load — the BSP's TSS is already "busy" and can't be shared.
    ; TODO: per-AP TSS allocation

    ; load kernel IDT
    lidt [trampoline.kernel_idt]

    ; serial 'I' = IDT loaded
    mov al, 0x49
    out dx, al

    ; serial 'S' = segments loaded
    mov al, 0x53
    mov dx, 0x3F8
    out dx, al

    mov byte [0x8008], 11
    mov qword [trampoline.ready], 0xFF

    ; serial 'J' = about to jump to Rust
    mov al, 0x4A
    mov dx, 0x3F8
    out dx, al

    ; RSP is 16-byte aligned. jmp to Rust entry point.
    mov rax, [trampoline.entry_point]
    jmp rax

ALIGN 16
gdt32:
    dq 0                       ; 0x00: null
    dq 0x00AF9A000000FFFF      ; 0x08: 64-bit code (matches kernel CS=0x08)
    dq 0x00AF92000000FFFF      ; 0x10: 64-bit data
    dq 0x00CF9A000000FFFF      ; 0x18: 32-bit code (mode transition only)
    dq 0x00CF92000000FFFF      ; 0x20: 32-bit data (mode transition only)
gdtr32:
    dw $ - gdt32 - 1
    dd gdt32
