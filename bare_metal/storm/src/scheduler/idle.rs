//! Idle loop: each CPU runs this, picking tasks and dispatching them.
//!
//! When no tasks are available, the CPU spins (Phase 1) or halts (Phase 3).

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Once;

use super::task::{TaskId, TaskState};
use super::SCHEDULER;
use crate::{arch, log, log_println};

/// Per-CPU idle state, accessed from both the idle loop and yield_current.
struct PerCpuIdleState {
    /// Pointer to the idle loop's saved RSP (written by dispatch, read by yield).
    idle_rsp_pointer: AtomicU64,
    /// Task currently running on this CPU (0 = none).
    current_task_id: AtomicU64,
}

/// Heap-allocated per-CPU idle state. Initialized by `init()`.
static PER_CPU_IDLE: Once<Vec<PerCpuIdleState>> = Once::new();

/// Initialize per-CPU idle state for the given number of CPU slots.
/// Must be called once after SMP discovery, before any CPU enters the idle loop.
pub fn init(cpu_count: usize) {
    PER_CPU_IDLE.call_once(|| {
        let mut v = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count {
            v.push(PerCpuIdleState {
                idle_rsp_pointer: AtomicU64::new(0),
                current_task_id: AtomicU64::new(0),
            });
        }
        v
    });
}

fn per_cpu(cpu_id: usize) -> &'static PerCpuIdleState {
    &PER_CPU_IDLE.get().expect("per-CPU idle state not initialized")[cpu_id]
}

/// Get the idle RSP for a CPU (used by yield_current to switch back to idle).
pub fn get_idle_rsp(cpu_id: usize) -> u64 {
    per_cpu(cpu_id).idle_rsp_pointer.load(Ordering::Acquire)
}

/// Get the current task ID running on a CPU (0 = none).
pub fn get_current_task_id(cpu_id: usize) -> Option<TaskId> {
    let id = per_cpu(cpu_id).current_task_id.load(Ordering::Acquire);
    if id == 0 { None } else { Some(id) }
}

/// Idle loop for one CPU. Call from ap_entry (APs) and kernel_main (BSP).
/// Never returns.
pub fn run_on_cpu(cpu_id: u64) -> ! {
    let cpu_id = cpu_id as usize;
    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CPU {} entering scheduler idle loop", cpu_id);

    let mut idle_rsp: u64 = 0;

    loop {
        let picked = {
            let mut state = SCHEDULER.lock();
            state.pick_next()
        };

        if let Some(task_id) = picked {
            dispatch(cpu_id, task_id, &mut idle_rsp);

            // Back from task. Re-enqueue if still Running (yield case).
            // block_current/exit_current set the state before returning here.
            let returned_id = per_cpu(cpu_id).current_task_id.swap(0, Ordering::Release);
            if returned_id != 0 {
                let mut state = SCHEDULER.lock();
                if let Some(task) = state.get(returned_id) {
                    if task.state == TaskState::Running {
                        state.requeue(returned_id);
                    }
                }
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

/// Dispatch a task on this CPU: context-switch to it and return when it
/// yields, gets preempted, blocks, or exits.
fn dispatch(cpu_id: usize, task_id: TaskId, idle_rsp: &mut u64) {
    let saved_rsp = {
        let mut state = SCHEDULER.lock();
        let task = match state.get_mut(task_id) {
            Some(t) => t,
            None => return,
        };
        task.last_cpu = Some(cpu_id as u32);
        task.saved_rsp
    };

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CPU {} dispatching task {} (rsp={:#x})", cpu_id, task_id, saved_rsp);

    // Publish per-CPU state so yield_current can find us
    let state = per_cpu(cpu_id);
    state.idle_rsp_pointer.store(idle_rsp as *mut u64 as u64, Ordering::Release);
    state.current_task_id.store(task_id, Ordering::Release);

    // Save idle RSP and switch to the task
    unsafe {
        arch::context_switch(idle_rsp as *mut u64, saved_rsp);
    }

    // Returned: task yielded, was preempted, blocked, or exited.
}
