//! Syscall handler for the Storm microkernel.
//!
//! Sets up the x86_64 SYSCALL/SYSRET mechanism via MSRs and provides a
//! naked assembly entry point that switches to the kernel stack, saves
//! user state, dispatches to a Rust handler, and returns via SYSRETQ.
//!
//! Syscall ABI (matches library_chaos):
//!   rax = syscall number
//!   rdi = arg1, rsi = arg2, rdx = arg3, r10 = arg4
//!   Returns: rax = result
//!
//! SYSCALL clobbers rcx (user RIP) and r11 (user RFLAGS).

use core::sync::atomic::{AtomicU64, Ordering};

use crate::gdt;
use crate::{log, log_println};

/// Kernel stack pointer for the process currently in user mode.
/// Set before SYSRETQ, read by the syscall entry to switch stacks.
/// TODO: make per-CPU when multiple CPUs run user code.
static KERNEL_RSP: AtomicU64 = AtomicU64::new(0);

/// Scratch space for saving user RSP during syscall entry.
static mut SCRATCH_RSP: u64 = 0;

/// Initialize the SYSCALL/SYSRET MSRs. Call once on BSP after GDT is loaded.
/// APs should also call this (MSRs are per-CPU).
pub fn init() {
    use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask};
    use x86_64::registers::rflags::RFlags;

    // Enable the SCE (System Call Extensions) bit in EFER
    unsafe { Efer::update(|flags| *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS) };

    // STAR MSR: encodes the CS/SS selectors for SYSCALL and SYSRET.
    // SYSRET loads: CS = cs_sysret (0x28|3 for 64-bit), SS = ss_sysret (0x20|3)
    // SYSCALL loads: CS = cs_syscall (0x08), SS = ss_syscall (0x10)
    Star::write(
        x86_64::structures::gdt::SegmentSelector::new(5, x86_64::PrivilegeLevel::Ring3),  // 0x2B user code 64
        x86_64::structures::gdt::SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3),  // 0x23 user data
        x86_64::structures::gdt::SegmentSelector::new(1, x86_64::PrivilegeLevel::Ring0),  // 0x08 kernel code
        x86_64::structures::gdt::SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0),  // 0x10 kernel data
    ).expect("Failed to write STAR MSR");

    // LSTAR: entry point for SYSCALL
    LStar::write(x86_64::VirtAddr::new(syscall_entry as *const () as u64));

    // SFMASK: clear IF on syscall entry (disable interrupts)
    SFMask::write(RFlags::INTERRUPT_FLAG);

    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
        "SYSCALL MSRs configured (entry={:#x})", syscall_entry as *const () as u64);
}

/// Set the kernel RSP that the syscall entry will switch to.
/// Must be called before entering user mode.
pub fn set_kernel_rsp(rsp: u64) {
    KERNEL_RSP.store(rsp, Ordering::Release);
}

/// Naked syscall entry point. The CPU arrives here with:
///   rcx = user RIP, r11 = user RFLAGS, rsp = user stack
///   rax = syscall number, rdi/rsi/rdx/r10 = args
///   IF is cleared by SFMASK
#[unsafe(naked)]
extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        // save user RSP to scratch, load kernel RSP
        "mov [rip + {scratch}], rsp",
        "mov rsp, [rip + {kernel_rsp}]",

        // build a frame on the kernel stack with all user state
        "push [rip + {scratch}]",   // user RSP
        "push rcx",                  // user RIP (saved by SYSCALL)
        "push r11",                  // user RFLAGS (saved by SYSCALL)

        // save callee-saved + arg registers (so Rust handler can read args)
        "push rax",                  // syscall number
        "push rdi",                  // arg1
        "push rsi",                  // arg2
        "push rdx",                  // arg3
        "push r10",                  // arg4
        "push r8",
        "push r9",
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // call Rust handler: rdi = syscall number, rsi = arg1, rdx = arg2,
        // rcx = arg3, r8 = arg4
        "mov rdi, [rsp + 13*8]",    // syscall number (rax, pushed 13 slots ago)
        "mov rsi, [rsp + 12*8]",    // arg1 (rdi)
        "mov rdx, [rsp + 11*8]",    // arg2 (rsi)
        "mov rcx, [rsp + 10*8]",    // arg3 (rdx)
        "mov r8,  [rsp +  9*8]",    // arg4 (r10)
        "call {handler}",

        // rax = return value from handler

        // restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "pop r9",
        "pop r8",
        "pop r10",
        "add rsp, 8",               // skip rdx
        "add rsp, 8",               // skip rsi
        "add rsp, 8",               // skip rdi
        "add rsp, 8",               // skip rax (return value already in rax)

        // restore user RIP, RFLAGS, RSP
        "pop r11",                   // user RFLAGS
        "pop rcx",                   // user RIP
        "pop rsp",                   // user RSP

        "sysretq",

        scratch = sym SCRATCH_RSP,
        kernel_rsp = sym KERNEL_RSP,
        handler = sym syscall_handler,
    );
}

/// Rust syscall dispatcher. Called from the naked entry with:
///   number = syscall number, arg1-arg4 = arguments
/// Returns a u64 result in rax.
extern "C" fn syscall_handler(number: u64, arg1: u64, arg2: u64, arg3: u64, _arg4: u64) -> u64 {
    match number {
        // ProcessEmit: arg1=emit_type, arg2=text_ptr, arg3=text_len
        401 => {
            let emit_type = arg1;
            let text_pointer = arg2 as *const u8;
            let text_length = arg3 as usize;

            // emit_type 0 = exit request
            if emit_type == 0 {
                log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
                    "Process exited with code {}", arg2);
                // For now, halt — the kernel thread that launched us will
                // never get control back. This is fine for the initial test.
                loop { core::hint::spin_loop(); }
            }

            // Safety: we trust the user pointer for now (TODO: validate it's in user space)
            let message = unsafe {
                if text_length > 0 && !text_pointer.is_null() {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(text_pointer, text_length))
                } else {
                    ""
                }
            };

            let level = match emit_type {
                1 => log::LogLevel::Error,
                2 => log::LogLevel::Warning,
                3 => log::LogLevel::Information,
                4 => log::LogLevel::Debug,
                _ => log::LogLevel::Debug,
            };

            log_println!(log::SubSystem::Kernel, level, "[userspace] {}", message);
            0 // success
        }

        _ => {
            log_println!(log::SubSystem::Kernel, log::LogLevel::Error,
                "Unknown syscall {} (args: {:#x}, {:#x}, {:#x})", number, arg1, arg2, arg3);
            3 // StormError::NotImplemented
        }
    }
}