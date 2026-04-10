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

use crate::arch::gdt;
use crate::{log, log_println};

macro_rules! const_assert {
    ($cond:expr) => { const _: () = assert!($cond); };
}

/// Per-CPU state for syscall handling. Initialized by `init_per_cpu_state()`
/// after the number of CPUs is known. Indexed by LAPIC ID.
///
/// `current_process` and `current_thread` are raw pointers stored as u64
/// to avoid process/thread table lookups on every syscall. The pointed-to
/// objects are heap-allocated and outlive the per-CPU references (cleared
/// on thread exit/yield before the objects are freed).
struct PerCpuSyscallState {
    /// Kernel stack pointer for this CPU's current user thread.
    kernel_rsp: AtomicU64,
    /// Scratch space for saving user RSP during syscall entry.
    scratch_rsp: AtomicU64,
    /// Raw pointer to the Process running on this CPU (null if none).
    current_process: AtomicU64,
    /// Raw pointer to the Thread running on this CPU (null if none).
    current_thread: AtomicU64,
}

/// Global per-CPU state table. Allocated on the heap after SMP discovery.
/// The naked syscall entry accesses this via `PER_CPU_TABLE` and `PER_CPU_COUNT`.
static mut PER_CPU_TABLE: *mut PerCpuSyscallState = core::ptr::null_mut();
static PER_CPU_COUNT: AtomicU64 = AtomicU64::new(0);

/// Allocate the per-CPU syscall state table. Call once after determining the
/// number of CPUs (max LAPIC ID + 1).
pub fn init_per_cpu_state(cpu_count: usize) {
    let layout = alloc::alloc::Layout::array::<PerCpuSyscallState>(cpu_count).unwrap();
    let pointer = unsafe { alloc::alloc::alloc_zeroed(layout) as *mut PerCpuSyscallState };
    assert!(!pointer.is_null(), "failed to allocate per-CPU syscall state");
    // AtomicU64 zero-initialized = 0, which is the correct default for all fields
    unsafe { PER_CPU_TABLE = pointer; }
    PER_CPU_COUNT.store(cpu_count as u64, Ordering::Release);
}

fn per_cpu(cpu_id: usize) -> &'static PerCpuSyscallState {
    assert!((cpu_id as u64) < PER_CPU_COUNT.load(Ordering::Acquire), "cpu_id out of range");
    unsafe { &*PER_CPU_TABLE.add(cpu_id) }
}

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

/// Set the kernel RSP for the current CPU (identified by LAPIC ID).
/// Must be called before entering user mode on this CPU.
pub fn set_kernel_rsp(cpu_id: usize, rsp: u64) {
    per_cpu(cpu_id).kernel_rsp.store(rsp, Ordering::Release);
}

/// Record which process and thread are running on this CPU.
/// Must be called before entering user mode. The pointers must remain
/// valid until the thread exits or yields (caller's responsibility).
pub fn set_current_context(
    cpu_id: usize,
    process: &crate::process::Process,
    thread: &crate::process::Thread,
) {
    let state = per_cpu(cpu_id);
    state.current_process.store(process as *const _ as u64, Ordering::Release);
    state.current_thread.store(thread as *const _ as u64, Ordering::Release);
}

/// Clear the current process/thread on this CPU (e.g. on thread exit).
pub fn clear_current_context(cpu_id: usize) {
    let state = per_cpu(cpu_id);
    state.current_process.store(0, Ordering::Release);
    state.current_thread.store(0, Ordering::Release);
}

/// Get the Process currently running on this CPU.
fn current_process() -> Option<&'static crate::process::Process> {
    let cpu_id = crate::arch::cpu_id();
    let pointer = per_cpu(cpu_id).current_process.load(Ordering::Acquire);
    if pointer == 0 { return None; }
    unsafe { Some(&*(pointer as *const crate::process::Process)) }
}

/// Get the Thread currently running on this CPU.
fn current_thread() -> Option<&'static crate::process::Thread> {
    let cpu_id = crate::arch::cpu_id();
    let pointer = per_cpu(cpu_id).current_thread.load(Ordering::Acquire);
    if pointer == 0 { return None; }
    unsafe { Some(&*(pointer as *const crate::process::Thread)) }
}

/// Naked syscall entry point. The CPU arrives here with:
///   rcx = user RIP, r11 = user RFLAGS, rsp = user stack
///   rax = syscall number, rdi/rsi/rdx/r10 = args
///   IF is cleared by SFMASK
#[unsafe(naked)]
extern "C" fn syscall_entry() {
    // PerCpuSyscallState layout: kernel_rsp(0), scratch_rsp(8),
    // current_process(16), current_thread(24) — 32 bytes per entry.
    const_assert!(core::mem::size_of::<PerCpuSyscallState>() == 32);

    core::arch::naked_asm!(
        // Per-CPU state is in a heap-allocated table pointed to by
        // PER_CPU_TABLE. Each entry is 32 bytes, indexed by LAPIC ID.
        //
        // Strategy: push rax, rcx, r11 onto the user stack to free up
        // three scratch registers. Compute the per-CPU entry address,
        // then save/restore via field offsets within the entry.

        // Save SYSCALL-critical registers on user stack
        "push rax",                          // [rsp+16] syscall number
        "push rcx",                          // [rsp+8]  user RIP
        "push r11",                          // [rsp+0]  user RFLAGS

        // Compute per-CPU entry address:
        //   entry = PER_CPU_TABLE + (lapic_id * 32)
        "mov eax, dword ptr [0xFEE00020]",  // LAPIC ID register
        "shr eax, 24",                       // eax = cpu_id
        "shl eax, 5",                       // eax = cpu_id * 32 (entry size)
        "mov rcx, [rip + {table}]",          // rcx = PER_CPU_TABLE base pointer
        "add rcx, rax",                      // rcx = &per_cpu_state[cpu_id]

        // Save original user RSP to entry.scratch_rsp (offset 8)
        "lea r11, [rsp + 24]",              // original user RSP (before 3 pushes)
        "mov [rcx + 8], r11",               // entry.scratch_rsp = user_rsp

        // Switch to per-CPU kernel stack from entry.kernel_rsp (offset 0)
        "mov rsp, [rcx]",

        // Now on kernel stack. Recover user state from user stack via
        // the scratch_rsp we just saved.
        "mov rcx, [rcx + 8]",               // rcx = original user RSP

        // Read saved values from user stack and push onto kernel stack.
        // User stack layout (push rax first, then rcx, then r11):
        //   [user_rsp -  8] = rax (syscall number)
        //   [user_rsp - 16] = rcx (user RIP)
        //   [user_rsp - 24] = r11 (user RFLAGS)
        "push rcx",                          // user RSP
        "mov r11, [rcx - 24]",              // saved r11 = user RFLAGS
        "mov rax, [rcx - 8]",               // saved rax = syscall number
        "mov rcx, [rcx - 16]",              // saved rcx = user RIP (must be last — clobbers rcx)

        "push rcx",                          // user RIP (saved by SYSCALL)
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
        // stack layout (16 pushes): r15(0) r14(1) r13(2) r12(3) rbp(4) rbx(5)
        //   r9(6) r8(7) r10(8) rdx(9) rsi(10) rdi(11) rax(12) r11(13) rcx(14) user_rsp(15)
        "mov rdi, [rsp + 12*8]",    // syscall number (rax)
        "mov rsi, [rsp + 11*8]",    // arg1 (rdi)
        "mov rdx, [rsp + 10*8]",    // arg2 (rsi)
        "mov rcx, [rsp +  9*8]",    // arg3 (rdx)
        "mov r8,  [rsp +  8*8]",    // arg4 (r10)
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

        table = sym PER_CPU_TABLE,
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
                // Switch back to the kernel's page tables and enter the
                // scheduler idle loop, freeing this CPU for other work.
                // The launch_user_process thread's stack is gone (we switched
                // RSP in the syscall entry), so we can't return normally.
                // Instead, load the kernel CR3 and jump directly into the
                // scheduler on this CPU.
                unsafe {
                    let kernel_cr3 = crate::arch::page_tables::get_kernel_cr3();
                    let cpu_id = crate::scheduler::read_lapic_id() as u64;
                    core::arch::asm!(
                        "mov cr3, {}",
                        in(reg) kernel_cr3,
                        options(nostack),
                    );
                    crate::scheduler::run_on_cpu(cpu_id);
                }
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