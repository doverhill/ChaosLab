//! Cooperative kernel thread scheduler.
//!
//! Each CPU runs an idle loop that picks threads from a global run queue.
//! Threads yield cooperatively via `scheduler::yield_thread()` which
//! context-switches back to the CPU's idle loop. The idle loop then picks
//! the next thread (or halts if none available).
//!
//! Preemptive switching via APIC timer will be added later, only when
//! there are more runnable threads than CPUs.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::{log, log_println, virtual_memory};

const THREAD_STACK_PAGES: usize = 16;
const MAX_CPUS: usize = 16;

static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

/// Per-CPU saved RSP for the idle loop. When a thread yields, it
/// context-switches back to this RSP.
static IDLE_RSP: Mutex<[u64; MAX_CPUS]> = Mutex::new([0; MAX_CPUS]);

/// Per-CPU currently running thread.
static CURRENT_THREAD: Mutex<[Option<Box<Thread>>; MAX_CPUS]> = Mutex::new([
    None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None,
]);

/// A kernel thread.
pub struct Thread {
    pub thread_id: u64,
    pub saved_rsp: u64,
    pub stack_base: u64,
    pub cpu_id: u64,
}

/// Global run queue.
pub static RUN_QUEUE: Mutex<VecDeque<Box<Thread>>> = Mutex::new(VecDeque::new());

pub type ThreadFunction = fn(u64) -> !;

/// Spawn a new kernel thread. It won't run until a CPU picks it up.
pub fn spawn(function: ThreadFunction, argument: u64) -> u64 {
    let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);

    let stack_base = virtual_memory::allocate_contiguous_pages(THREAD_STACK_PAGES)
        .expect("Failed to allocate thread stack") as u64;
    let stack_top = stack_base + (THREAD_STACK_PAGES * 0x1000) as u64;

    // Initial stack for context_switch to restore into thread_bootstrap:
    // context_switch pops: r15, r14, r13, r12, rbx, rbp, then ret
    unsafe {
        let base = stack_top as *mut u64;
        *base.offset(-1) = 0;                                             // alignment
        *base.offset(-2) = function as u64;                               // for bootstrap
        *base.offset(-3) = argument;                                      // for bootstrap
        *base.offset(-4) = thread_bootstrap as *const () as u64;          // ret target
        *base.offset(-5) = 0;  // r15
        *base.offset(-6) = 0;  // r14
        *base.offset(-7) = 0;  // r13
        *base.offset(-8) = 0;  // r12
        *base.offset(-9) = 0;  // rbx
        *base.offset(-10) = 0; // rbp
    }

    let thread = Box::new(Thread {
        thread_id,
        saved_rsp: stack_top - 80,
        stack_base,
        cpu_id: 0,
    });

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "Spawned kernel thread {}", thread_id);

    RUN_QUEUE.lock().push_back(thread);
    thread_id
}

/// Bootstrap for new threads. context_switch `ret`s here.
/// Must be naked — the stack has argument and function pointer that
/// we need to pop before any compiler-generated prologue runs.
#[unsafe(naked)]
extern "C" fn thread_bootstrap() -> ! {
    core::arch::naked_asm!(
        "pop rdi",          // argument (first parameter in System V ABI)
        "pop rax",          // function pointer
        "call rax",         // call the thread function (never returns)
        "ud2",              // unreachable
    );
}

/// Idle loop for each CPU. Call this from ap_entry (and BSP after init).
pub fn run_on_cpu(cpu_id: u64) -> ! {
    loop {
        // check for a runnable thread
        let thread = RUN_QUEUE.lock().pop_front();
        if let Some(mut thread) = thread {
            thread.cpu_id = cpu_id;
            let thread_rsp = thread.saved_rsp;

            // store the thread as the current thread for this CPU
            CURRENT_THREAD.lock()[cpu_id as usize] = Some(thread);

            // context switch: save idle RSP, jump to thread
            let idle_rsp_ptr = {
                let mut idle = IDLE_RSP.lock();
                &mut idle[cpu_id as usize] as *mut u64
            };
            unsafe { context_switch(idle_rsp_ptr, thread_rsp) };

            // we return here when the thread yields
            // the thread has already been moved back to the run queue by yield_thread
        } else {
            // no threads — brief pause then check again
            // TODO: use hlt + IPI to wake CPUs when threads are added
            core::hint::spin_loop();
        }
    }
}

/// Yield the current thread. Saves its state and switches back to the
/// CPU's idle loop. The thread is put back in the run queue.
pub fn yield_thread() {
    x86_64::instructions::interrupts::disable();

    // figure out which CPU we're on by scanning CURRENT_THREAD
    let (cpu_id, idle_rsp) = {
        let current = CURRENT_THREAD.lock();
        let cpu_id = current.iter().position(|t| t.is_some()).unwrap_or(0);
        let idle = IDLE_RSP.lock();
        (cpu_id, idle[cpu_id])
    };

    // move the current thread back to the run queue
    let thread = CURRENT_THREAD.lock()[cpu_id].take();
    if let Some(mut thread) = thread {
        // we'll save our RSP into the thread's saved_rsp via context_switch
        let thread_rsp_ptr = &mut thread.saved_rsp as *mut u64;
        RUN_QUEUE.lock().push_back(thread);

        // context switch back to idle loop
        unsafe { context_switch(thread_rsp_ptr, idle_rsp) };
        // we resume here when the idle loop picks us up again
    }

    x86_64::instructions::interrupts::enable();
}

/// Low-level context switch. Saves callee-saved registers + RSP, loads new RSP + registers.
#[unsafe(naked)]
extern "C" fn context_switch(_old_rsp_ptr: *mut u64, _new_rsp: u64) {
    core::arch::naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",   // save current RSP to *old_rsp_ptr
        "mov rsp, rsi",     // load new RSP
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}
