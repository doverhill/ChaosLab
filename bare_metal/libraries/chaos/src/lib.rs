//! Bare-metal Chaos system library.
//!
//! Provides the runtime for user-space applications on the Storm microkernel:
//! entry point, panic handler, heap allocator, and syscall wrappers.
//!
//! Applications link against this crate and only need to provide:
//! ```rust
//! #![no_std]
//! #![no_main]
//! extern crate library_chaos;
//! extern crate alloc;
//!
//! #[no_mangle]
//! pub fn chaos_main() {
//!     // your code here — heap allocation works, process_emit works
//! }
//! ```
//!
//! If `chaos_main` returns, the process exits with code 0.
//! If it panics, the panic message is logged and the process exits with code 1.

#![no_std]

extern crate alloc;

use core::fmt::Write;

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
pub const SYSCALL_THREAD_CREATE: u64 = 700;
pub const SYSCALL_THREAD_DESTROY: u64 = 701;
pub const SYSCALL_MEMORY_ALLOCATE: u64 = 800;
pub const SYSCALL_MEMORY_FREE: u64 = 801;
pub const SYSCALL_MEMORY_MAP: u64 = 802;
pub const SYSCALL_MEMORY_UNMAP: u64 = 803;
pub const SYSCALL_INTERRUPT_CREATE: u64 = 900;
pub const SYSCALL_INTERRUPT_DESTROY: u64 = 901;
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
// High-level syscall wrappers
// ---------------------------------------------------------------------------

/// Emit a log message to the kernel console.
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
        syscall(SYSCALL_PROCESS_EMIT, 0, code, 0, 0);
    }
    loop { core::hint::spin_loop(); }
}

/// Allocate `page_count` pages of virtual memory.
/// Returns the base virtual address, or None on failure.
pub fn memory_allocate(page_count: usize) -> Option<*mut u8> {
    let result = unsafe { syscall(SYSCALL_MEMORY_ALLOCATE, page_count as u64, 0, 0, 0) };
    if result == 0 { None } else { Some(result as *mut u8) }
}

/// Free `page_count` pages of virtual memory starting at `address`.
pub fn memory_free(address: *mut u8, page_count: usize) {
    unsafe { syscall(SYSCALL_MEMORY_FREE, address as u64, page_count as u64, 0, 0); }
}

// ---------------------------------------------------------------------------
// Heap allocator — uses MemoryAllocate/MemoryFree syscalls as backing
// ---------------------------------------------------------------------------

const PAGE_SIZE: usize = 4096;
const MIN_HEAP_GROWTH_PAGES: usize = 16; // 64 KiB minimum growth

/// Source that grows the heap by requesting pages from the kernel via syscalls.
#[derive(Debug)]
struct SyscallMemorySource;

unsafe impl talc::source::Source for SyscallMemorySource {
    fn acquire<B: talc::base::binning::Binning>(
        talc: &mut talc::base::Talc<Self, B>,
        layout: core::alloc::Layout,
    ) -> Result<(), ()> {
        let needed_bytes = layout.size().max(layout.align());
        let pages = ((needed_bytes + PAGE_SIZE - 1) / PAGE_SIZE).max(MIN_HEAP_GROWTH_PAGES);

        let base = memory_allocate(pages).ok_or(())?;
        let size = pages * PAGE_SIZE;

        unsafe {
            if talc.claim(base, size).is_none() {
                // region too small — shouldn't happen with MIN_HEAP_GROWTH_PAGES
                memory_free(base, pages);
                return Err(());
            }
        }
        Ok(())
    }
}

#[global_allocator]
static ALLOCATOR: talc::TalcLock<spin::Mutex<()>, SyscallMemorySource> =
    talc::TalcLock::new(SyscallMemorySource);

// ---------------------------------------------------------------------------
// Entry point and panic handler — the Chaos app runtime
// ---------------------------------------------------------------------------

extern "Rust" {
    /// The application's main function. Generated by the `main!` macro.
    fn chaos_main();
}

/// Process entry point. Sets up the runtime, calls the app's `chaos_main`,
/// and exits cleanly if it returns.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe { chaos_main(); }
    process_exit(0);
}

/// Define the application's entry point. This generates the required
/// `chaos_main` symbol that the library_chaos runtime calls.
///
/// Usage:
/// ```rust
/// library_chaos::main! {
///     // your code here — heap, process_emit, etc. all work
/// }
/// ```
#[macro_export]
macro_rules! main {
    ($($body:tt)*) => {
        #[no_mangle]
        pub fn chaos_main() {
            $($body)*
        }
    };
}

/// Panic handler — logs the panic message via ProcessEmit and exits.
/// Uses a small stack buffer to format without heap (the heap may be
/// the thing that panicked).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Format into a stack buffer to avoid heap allocation in the panic path
    let mut buffer = StackWriter::new();
    let _ = write!(buffer, "PANIC: {}", info);
    process_emit(EmitType::Error, buffer.as_str());
    process_exit(1);
}

/// Small fixed-size buffer for formatting panic messages without heap.
struct StackWriter {
    buffer: [u8; 512],
    position: usize,
}

impl StackWriter {
    fn new() -> Self {
        Self { buffer: [0; 512], position: 0 }
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.position]) }
    }
}

impl core::fmt::Write for StackWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buffer.len() - self.position;
        let to_copy = bytes.len().min(remaining);
        self.buffer[self.position..self.position + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.position += to_copy;
        Ok(())
    }
}
