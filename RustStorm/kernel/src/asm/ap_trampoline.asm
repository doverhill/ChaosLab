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
    .ready:       dq 0       ; offset 0x08 — debug stage counter
    .page_table:  dq 0       ; offset 0x10
    .stack_top:   dq 0       ; offset 0x18
    .entry_point: dq 0       ; offset 0x20


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

    jmp dword 0x08:protected_mode

BITS 32
protected_mode:
    mov byte [0x8008], 4

    mov ax, 0x10
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

    jmp 0x18:long_mode

BITS 64
long_mode:
    mov byte [0x8008], 10

    ; load null data segments — 64-bit mode doesn't use DS/ES/SS bases,
    ; and null selectors are always valid. This avoids conflicts when
    ; the kernel loads its own GDT (which has different segment layout).
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov rsp, [trampoline.stack_top]

    ; enable SSE: clear CR0.EM (bit 2), set CR0.MP (bit 1)
    mov rax, cr0
    and ax, 0xFFFB      ; clear EM
    or ax, 0x0002       ; set MP
    mov cr0, rax

    ; set CR4.OSFXSR (bit 9) and CR4.OSXMMEXCPT (bit 10)
    mov rax, cr4
    or ax, (1 << 9) | (1 << 10)
    mov cr4, rax

    mov byte [0x8008], 11

    mov qword [trampoline.ready], 0xFF

    mov rax, [trampoline.entry_point]
    call rax

    ; should never return, but just in case
.halt_forever:
    hlt
    jmp .halt_forever

ALIGN 16
gdt32:
    dq 0
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
    dq 0x00AF9A000000FFFF
    dq 0x00AF92000000FFFF
gdtr32:
    dw $ - gdt32 - 1
    dd gdt32
