//! Scheduler: manages all executable work (kernel threads and user threads)
//! as unified Tasks with priority-based preemptive scheduling.
//!
//! Public API:
//! - `spawn_kernel(function, argument, priority)` — create a kernel task
//! - `spawn_user(process_id, ...)` — create a user thread task
//! - `yield_current()` — cooperative yield
//! - `block_current(reason)` — block until event/timeout
//! - `unblock(task_id)` — wake a blocked task
//! - `exit_current()` — terminate the current task
//! - `run_on_cpu(cpu_id)` — idle loop (called once per CPU)

pub mod task;
pub mod run_queue;
pub mod timer_queue;
pub mod state;
pub mod idle;
pub mod task_mutex;

use spin::Mutex;
use crate::{arch, log, log_println, virtual_memory};
use task::{Task, TaskId, TaskKind, TaskState};
use state::SchedulerState;

/// The global scheduler state (task table + run queue + timer queue).
pub static SCHEDULER: Mutex<SchedulerState> = Mutex::new(SchedulerState::new());

/// Kernel task stack size in pages (64 KiB).
const KERNEL_STACK_PAGES: usize = 16;

/// Function signature for kernel tasks.
pub type KernelTaskFunction = fn(u64) -> !;


// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Spawn a new kernel task. Returns the TaskId.
pub fn spawn_kernel(function: KernelTaskFunction, argument: u64, priority: i32) -> TaskId {
    let task_id = state::allocate_task_id();

    // Allocate a kernel stack
    let stack_base = virtual_memory::allocate_contiguous_pages(KERNEL_STACK_PAGES)
        .expect("failed to allocate kernel task stack") as u64;
    let stack_top = stack_base + (KERNEL_STACK_PAGES * 0x1000) as u64;

    // Set up the initial stack frame for context_switch → kernel_task_bootstrap.
    // context_switch pops: r15, r14, r13, r12, rbx, rbp, then ret.
    // kernel_task_bootstrap pops: argument, function_pointer, then call.
    unsafe {
        let base = stack_top as *mut u64;
        *base.offset(-1) = 0;                                             // alignment
        *base.offset(-2) = function as u64;                               // for bootstrap
        *base.offset(-3) = argument;                                      // for bootstrap
        *base.offset(-4) = arch::kernel_task_bootstrap as *const () as u64; // ret target
        *base.offset(-5) = 0;  // r15
        *base.offset(-6) = 0;  // r14
        *base.offset(-7) = 0;  // r13
        *base.offset(-8) = 0;  // r12
        *base.offset(-9) = 0;  // rbx
        *base.offset(-10) = 0; // rbp
    }

    let task = Task {
        task_id,
        state: TaskState::Runnable,
        kind: TaskKind::Kernel,
        priority,
        saved_rsp: stack_top - 80,
        kernel_stack_base: stack_base,
        kernel_stack_top: stack_top,
        user_stack_top: 0,
        entry_point: 0,
        cr3: 0,
        block_reason: None,
        cpu_affinity: None,
        last_cpu: None,
        ipc_partner: None,
        timeslice_remaining_ticks: 0,
    };

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "Spawned kernel task {}", task_id);

    SCHEDULER.lock().add_and_enqueue(task);
    idle::wake_idle_cpu();
    task_id
}

/// Yield the current task cooperatively. The idle loop will re-enqueue it.
pub fn yield_current() {
    x86_64::instructions::interrupts::disable();

    let cpu_id = arch::cpu_id();
    let task_id = match idle::get_current_task_id(cpu_id) {
        Some(id) => id,
        None => {
            x86_64::instructions::interrupts::enable();
            return; // no task running (shouldn't happen, but be safe)
        }
    };

    // Get the idle RSP to switch back to (dereference the pointer to the
    // idle loop's stack variable), and a pointer into the task's saved_rsp
    // field so context_switch can save our RSP there.
    let idle_rsp_pointer = idle::get_idle_rsp(cpu_id);
    if idle_rsp_pointer == 0 {
        // No idle loop set up for this CPU yet — can't yield
        x86_64::instructions::interrupts::enable();
        return;
    }
    let idle_rsp = unsafe { *(idle_rsp_pointer as *const u64) };
    let task_rsp_ptr = {
        let mut state = SCHEDULER.lock();
        match state.get_mut(task_id) {
            Some(task) => &mut task.saved_rsp as *mut u64,
            None => {
                x86_64::instructions::interrupts::enable();
                return;
            }
        }
    };
    // Lock is released before context_switch (critical — can't hold across switch)

    unsafe {
        arch::context_switch(task_rsp_ptr, idle_rsp);
    }

    // Resumed here after being dispatched again
    x86_64::instructions::interrupts::enable();
}

/// Block the current task. It will not be scheduled until unblock() is called
/// or its timeout expires.
pub fn block_current(_reason: task::BlockReason) {
    // TODO: Phase 1 stub. Will be implemented when EventWait syscall lands.
    yield_current();
}

/// Unblock a task, making it Runnable.
pub fn unblock(task_id: TaskId) {
    let unblocked = SCHEDULER.lock().unblock(task_id);
    if unblocked {
        idle::wake_idle_cpu();
    }
}

/// Terminate the current task.
pub fn exit_current() -> ! {
    // TODO: proper implementation. For now, restore kernel CR3 and re-enter idle.
    unsafe {
        let kernel_cr3 = crate::arch::page_tables::get_kernel_cr3();
        let cpu_id = arch::cpu_id() as u64;
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack));
        idle::run_on_cpu(cpu_id);
    }
}

/// Entry point for the idle loop on each CPU.
pub use idle::run_on_cpu;
