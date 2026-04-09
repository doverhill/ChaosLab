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

    lgdt [gdtr32]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp dword 0x18:protected_mode   ; 0x18 = 32-bit code

BITS 32
protected_mode:

    mov ax, 0x20                     ; 0x20 = 32-bit data
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    mov eax, [trampoline.page_table]
    mov cr3, eax

    ; enable long mode (LME) AND no-execute (NXE) in EFER
    ; NXE is critical: the BSP's page tables have NX bits set on data pages.
    ; Without NXE, bit 63 in page table entries becomes a reserved bit,
    ; causing page faults on every page walk through those entries.
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8) | (1 << 11)  ; LME + NXE
    wrmsr

    ; enable paging + write protect
    mov eax, cr0
    or eax, (1 << 31) | (1 << 16)  ; PG + WP
    mov cr0, eax

    jmp 0x08:long_mode               ; 0x08 = 64-bit code (matches kernel)

BITS 64
long_mode:

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

    ; Load data segments from trampoline GDT (0x10 = data segment).
    ; Both trampoline and kernel GDTs have a valid data segment at 0x10.
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; Load kernel GDT — CS=0x08 and data=0x10 match between GDTs,
    ; so no CS far-jump reload is needed.
    lgdt [trampoline.kernel_gdt]

    ; Load kernel IDT so exceptions produce diagnostics instead of triple faults.
    ; NOTE: no TSS loaded — IST disabled on exception handlers for now.
    ; TODO: per-AP TSS allocation
    lidt [trampoline.kernel_idt]

    mov qword [trampoline.ready], 0xFF

    ; System V ABI: RSP must be 16n+8 at function entry (simulates post-call).
    ; RSP is currently 16-aligned (stack_top). Subtract 8.
    sub rsp, 8
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
