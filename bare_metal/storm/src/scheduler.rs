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

/// Per-CPU state. Each element is independently locked to avoid contention.
struct CpuState {
    idle_rsp: u64,
    current_thread: Option<Box<Thread>>,
}

static CPU_STATE: [Mutex<CpuState>; MAX_CPUS] = {
    const INIT: Mutex<CpuState> = Mutex::new(CpuState { idle_rsp: 0, current_thread: None });
    [INIT; MAX_CPUS]
};

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

/// Read the current CPU's LAPIC ID from the xAPIC MMIO register.
/// This is the most reliable way to identify which CPU we're on.
pub fn read_lapic_id() -> u32 {
    // xAPIC ID register is at APIC base + 0x20, bits 31:24
    let apic_base: u64 = 0xFEE00000; // standard xAPIC base
    let id_register = unsafe { core::ptr::read_volatile((apic_base + 0x20) as *const u32) };
    id_register >> 24
}

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
    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CPU {} (LAPIC {}) entering scheduler idle loop", cpu_id, read_lapic_id());
    loop {
        let thread = RUN_QUEUE.lock().pop_front();
        if let Some(mut thread) = thread {
            thread.cpu_id = cpu_id;
            let thread_id = thread.thread_id;
            let thread_rsp = thread.saved_rsp;

            log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
                "CPU {} dispatching thread {} (rsp={:#x})", cpu_id, thread_id, thread_rsp);

            // store thread and get pointer to idle_rsp for saving
            let idle_rsp_ptr = {
                let mut state = CPU_STATE[cpu_id as usize].lock();
                state.current_thread = Some(thread);
                &mut state.idle_rsp as *mut u64
            };
            // lock is dropped here — safe to context switch

            context_switch(idle_rsp_ptr, thread_rsp);

            // thread yielded back — re-queue it now that context_switch
            // has saved the correct RSP into the thread
            let yielded = CPU_STATE[cpu_id as usize].lock().current_thread.take();
            if let Some(thread) = yielded {
                log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
                    "CPU {} re-queuing thread {}", cpu_id, thread.thread_id);
                RUN_QUEUE.lock().push_back(thread);
            } else {
                log_println!(log::SubSystem::Kernel, log::LogLevel::Error,
                    "CPU {} returned from context_switch but no thread in CPU_STATE!", cpu_id);
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

/// Yield the current thread. Saves its state and switches back to the
/// CPU's idle loop. The thread goes back into the run queue.
pub fn yield_thread() {
    x86_64::instructions::interrupts::disable();

    // use LAPIC ID to determine our CPU — reliable, no scanning needed
    let cpu_id = read_lapic_id() as usize;

    // get the RSP save pointer and idle RSP — leave the thread in CPU_STATE
    // so that the idle loop can re-queue it AFTER context_switch has saved RSP.
    // Moving the thread to the run queue before saving RSP is a race: another
    // CPU could pick it up with stale saved_rsp.
    let (idle_rsp, thread_rsp_ptr, thread_id) = {
        let mut state = CPU_STATE[cpu_id].lock();
        let idle_rsp = state.idle_rsp;
        let thread = state.current_thread.as_mut().expect("yield_thread: no current thread");
        let thread_id = thread.thread_id;
        let thread_rsp_ptr = &mut thread.saved_rsp as *mut u64;
        (idle_rsp, thread_rsp_ptr, thread_id)
    };
    // lock is dropped here — safe to context switch

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "Thread {} yielding on CPU {} (LAPIC {}), idle_rsp={:#x}", thread_id, cpu_id, cpu_id, idle_rsp);

    // context switch back to idle loop (saves our RSP via thread_rsp_ptr)
    context_switch(thread_rsp_ptr, idle_rsp);

    // we resume here when picked up again
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
