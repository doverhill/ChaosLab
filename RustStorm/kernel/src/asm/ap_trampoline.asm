; Application Processor trampoline
;
; This code is assembled to a flat binary and placed at physical address 0x8000.
; APs start executing here in 16-bit real mode after receiving a SIPI.
; It transitions: real mode -> protected mode -> long mode -> jump to Rust.
;
; The BSP patches the data fields before sending each SIPI.

ORG 0x8000

SECTION .text

; ---------------------------------------------------------------------------
; Data fields (mailbox, patched by BSP before each AP startup)
; ---------------------------------------------------------------------------
trampoline:
    jmp short startup_ap

    ; pad to offset 8
    times 8 - ($ - trampoline) nop

    .ready:       dq 0       ; offset 0x08: AP sets to 1 when trampoline is done
    .page_table:  dq 0       ; offset 0x10: CR3 value (kernel page table physical address)
    .stack_top:   dq 0       ; offset 0x18: per-AP stack top pointer
    .entry_point: dq 0       ; offset 0x20: Rust AP entry function pointer

; ---------------------------------------------------------------------------
; 16-bit real mode entry
; ---------------------------------------------------------------------------
BITS 16
startup_ap:
    cli
    cld

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; load the temporary GDT
    lgdt [gdtr32]

    ; enable protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; far jump to 32-bit code
    jmp 0x08:protected_mode

; ---------------------------------------------------------------------------
; 32-bit protected mode
; ---------------------------------------------------------------------------
BITS 32
protected_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; enable PAE
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ; load page table (CR3)
    mov eax, [trampoline.page_table]
    mov cr3, eax

    ; enable long mode via EFER.LME
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; enable paging
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    ; far jump to 64-bit code
    jmp 0x18:long_mode

; ---------------------------------------------------------------------------
; 64-bit long mode
; ---------------------------------------------------------------------------
BITS 64
long_mode:
    mov ax, 0x20
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor ax, ax
    mov fs, ax
    mov gs, ax

    ; load per-AP stack
    mov rsp, [trampoline.stack_top]

    ; signal that we are done with the trampoline page
    mov qword [trampoline.ready], 1

    ; jump to Rust AP entry point
    mov rax, [trampoline.entry_point]
    jmp rax

; ---------------------------------------------------------------------------
; Temporary GDT (only used during the mode transition)
; ---------------------------------------------------------------------------
ALIGN 16
gdt32:
    ; null descriptor
    dq 0
    ; 32-bit code segment (selector 0x08)
    dq 0x00CF9A000000FFFF
    ; 32-bit data segment (selector 0x10)
    dq 0x00CF92000000FFFF
    ; 64-bit code segment (selector 0x18)
    dq 0x00AF9A000000FFFF
    ; 64-bit data segment (selector 0x20)
    dq 0x00AF92000000FFFF

gdtr32:
    dw $ - gdt32 - 1    ; limit
    dd gdt32             ; base (32-bit, works in real/protected mode)
