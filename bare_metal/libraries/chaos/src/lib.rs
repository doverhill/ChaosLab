//! Bare-metal Chaos system library.
//!
//! Provides syscall wrappers for user-space applications running on the
//! Storm microkernel. Syscall numbers match the hosted library in
//! `Library/Chaos/src/syscalls.rs`.

#![no_std]

// ---------------------------------------------------------------------------
// Syscall numbers (must match kernel's syscall handler)
// ---------------------------------------------------------------------------

pub const SYSCALL_SERVICE_CREATE: u64 = 100;
pub const SYSCALL_SERVICE_SUBSCRIBE: u64 = 101;
pub const SYSCALL_CHANNEL_SIGNAL: u64 = 200;
pub const SYSCALL_EVENT_WAIT: u64 = 300;
pub const SYSCALL_PROCESS_CREATE: u64 = 400;
pub const SYSCALL_PROCESS_EMIT: u64 = 401;
pub const SYSCALL_PROCESS_REDUCE_CAPABILITIES: u64 = 402;
pub const SYSCALL_TIMER_CREATE: u64 = 500;
pub const SYSCALL_QUERY: u64 = 600;
pub const SYSCALL_HANDLE_DESTROY: u64 = 1000;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum StormError {
    None = 0,
    NotFound = 1,
    PermissionDenied = 2,
    NotImplemented = 3,
    Timeout = 4,
    Cancelled = 5,
    AlreadyExists = 6,
    General = 7,
    Malformed = 8,
}

// ---------------------------------------------------------------------------
// Emit types (for ProcessEmit syscall)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum EmitType {
    Error = 1,
    Warning = 2,
    Information = 3,
    Debug = 4,
}

// ---------------------------------------------------------------------------
// Raw syscall interface
// ---------------------------------------------------------------------------

/// Invoke a syscall with up to 4 arguments.
///
/// ABI: rax=number, rdi=arg1, rsi=arg2, rdx=arg3, r10=arg4
/// Returns: rax (result/error code)
///
/// Note: `syscall` clobbers rcx (saves RIP) and r11 (saves RFLAGS).
#[inline(always)]
pub unsafe fn syscall(number: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let result: u64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") number => result,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    result
}

// ---------------------------------------------------------------------------
// High-level wrappers
// ---------------------------------------------------------------------------

/// Emit a log message to the kernel console.
///
/// This is the simplest syscall — the kernel prints the message to its
/// serial/framebuffer log. Used for testing that userspace → kernel
/// communication works.
pub fn process_emit(emit_type: EmitType, message: &str) {
    unsafe {
        syscall(
            SYSCALL_PROCESS_EMIT,
            emit_type as u64,
            message.as_ptr() as u64,
            message.len() as u64,
            0,
        );
    }
}

/// Exit the current process.
pub fn process_exit(code: u64) -> ! {
    unsafe {
        // Use a dedicated exit that the kernel recognizes.
        // For now, reuse ProcessEmit with a special emit_type=0 as "exit".
        // TODO: proper ProcessDestroy syscall
        syscall(SYSCALL_PROCESS_EMIT, 0, code, 0, 0);
    }
    loop {
        core::hint::spin_loop();
    }
}
