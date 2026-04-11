//! Idle loop: each CPU runs this, picking tasks and dispatching them.
//!
//! When no tasks are available, the CPU halts (HLT) until woken by an
//! IPI from another CPU or a timer interrupt. Idle time is tracked per
//! CPU via TSC for utilization metrics.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Once;

use super::task::{TaskId, TaskState};
use super::SCHEDULER;
use crate::{arch, log, log_println};

// ---------------------------------------------------------------------------
// Per-CPU idle state
// ---------------------------------------------------------------------------

struct PerCpuIdleState {
    /// Pointer to the idle loop's saved RSP (written by dispatch, read by yield).
    idle_rsp_pointer: AtomicU64,
    /// Task currently running on this CPU (0 = none).
    current_task_id: AtomicU64,
    /// Whether this CPU is halted waiting for work.
    is_idle: AtomicBool,
    /// TSC when this CPU entered idle (0 = not idle).
    idle_since_tsc: AtomicU64,
    /// Accumulated idle TSC ticks for utilization measurement.
    total_idle_tsc: AtomicU64,
}

/// Heap-allocated per-CPU idle state. Initialized by `init()`.
static PER_CPU_IDLE: Once<Vec<PerCpuIdleState>> = Once::new();

/// Cached hint: a CPU ID that is known to be idle, or -1 (u64::MAX)
/// if unknown. Updated on begin_idle/end_idle. Avoids scanning the
/// per-CPU array on every wake_idle_cpu call.
const NO_IDLE_CPU: u64 = u64::MAX;
static FIRST_IDLE_CPU: AtomicU64 = AtomicU64::new(NO_IDLE_CPU);

/// Initialize per-CPU idle state for the given number of CPU slots.
/// Must be called once after SMP discovery, before any CPU enters the idle loop.
pub fn init(cpu_count: usize) {
    PER_CPU_IDLE.call_once(|| {
        let mut v = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count {
            v.push(PerCpuIdleState {
                idle_rsp_pointer: AtomicU64::new(0),
                current_task_id: AtomicU64::new(0),
                is_idle: AtomicBool::new(false),
                idle_since_tsc: AtomicU64::new(0),
                total_idle_tsc: AtomicU64::new(0),
            });
        }
        v
    });
}

fn per_cpu(cpu_id: usize) -> &'static PerCpuIdleState {
    &PER_CPU_IDLE.get().expect("per-CPU idle state not initialized")[cpu_id]
}

// ---------------------------------------------------------------------------
// Public accessors (used by yield_current, timer handler, etc.)
// ---------------------------------------------------------------------------

/// Get the idle RSP for a CPU (used by yield_current to switch back to idle).
pub fn get_idle_rsp(cpu_id: usize) -> u64 {
    per_cpu(cpu_id).idle_rsp_pointer.load(Ordering::Acquire)
}

/// Get the current task ID running on a CPU (0 = none).
pub fn get_current_task_id(cpu_id: usize) -> Option<TaskId> {
    let id = per_cpu(cpu_id).current_task_id.load(Ordering::Acquire);
    if id == 0 { None } else { Some(id) }
}

/// Wake an idle CPU by sending a scheduler IPI. If no CPU is idle, does nothing.
/// Called after making a task runnable (spawn, unblock, etc.).
pub fn wake_idle_cpu() {
    // Fast path: check the cached hint
    let hint = FIRST_IDLE_CPU.load(Ordering::Acquire);
    if hint != NO_IDLE_CPU {
        arch::send_scheduler_ipi(hint as usize);
        return;
    }
    // Slow path: scan per-CPU state (hint was stale)
    if let Some(per_cpu_vec) = PER_CPU_IDLE.get() {
        for (id, state) in per_cpu_vec.iter().enumerate() {
            if state.is_idle.load(Ordering::Acquire) {
                arch::send_scheduler_ipi(id);
                return;
            }
        }
    }
    // No idle CPUs — the task will be picked up when a CPU finishes its current work
}

// ---------------------------------------------------------------------------
// Idle loop
// ---------------------------------------------------------------------------

/// Idle loop for one CPU. Call from ap_entry (APs) and kernel_main (BSP).
/// Never returns.
pub fn run_on_cpu(cpu_id: u64) -> ! {
    let cpu_id = cpu_id as usize;
    super::task_mutex::set_scheduler_active();

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CPU {} entering scheduler idle loop", cpu_id);

    let mut idle_rsp: u64 = 0;

    loop {
        let picked = {
            let mut state = SCHEDULER.lock();
            state.pick_next()
        };

        if let Some(task_id) = picked {
            end_idle(cpu_id);
            dispatch(cpu_id, task_id, &mut idle_rsp);

            // Re-enqueue if still Running (yield/preempt case).
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
            // Nothing to run — halt until woken by IPI or timer interrupt.
            begin_idle(cpu_id);
            arch::halt_until_interrupt();
            end_idle(cpu_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Idle metrics
// ---------------------------------------------------------------------------

fn begin_idle(cpu_id: usize) {
    let state = per_cpu(cpu_id);
    state.is_idle.store(true, Ordering::Release);
    state.idle_since_tsc.store(arch::read_tsc(), Ordering::Relaxed);
    // Update the hint — this CPU is available
    FIRST_IDLE_CPU.store(cpu_id as u64, Ordering::Release);
}

fn end_idle(cpu_id: usize) {
    let state = per_cpu(cpu_id);
    state.is_idle.store(false, Ordering::Release);
    // Invalidate the hint if it pointed to us
    let _ = FIRST_IDLE_CPU.compare_exchange(
        cpu_id as u64, NO_IDLE_CPU, Ordering::Release, Ordering::Relaxed,
    );
    // Accumulate idle time
    let started = state.idle_since_tsc.swap(0, Ordering::Relaxed);
    if started != 0 {
        let elapsed = arch::read_tsc().saturating_sub(started);
        state.total_idle_tsc.fetch_add(elapsed, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

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

    // Publish per-CPU state so yield_current and the timer handler can find us
    let per_cpu_state = per_cpu(cpu_id);
    per_cpu_state.idle_rsp_pointer.store(idle_rsp as *mut u64 as u64, Ordering::Release);
    per_cpu_state.current_task_id.store(task_id, Ordering::Release);

    // Arm the APIC timer for preemption if there are more tasks waiting.
    // Dynamic ticks: only arm when there's contention for this CPU.
    // FIXME: with per-CPU local run queues, the next task to run would
    // already be staged here — the timer handler could switch directly
    // to it without going through the idle loop and locking the global
    // scheduler. That eliminates one context switch per preemption.
    let has_pending = !SCHEDULER.lock().run_queue.is_empty();
    if has_pending {
        arch::timer::arm_apic_timer_milliseconds(5); // 5ms timeslice
    }

    // Save idle RSP and switch to the task.
    unsafe {
        arch::context_switch(idle_rsp as *mut u64, saved_rsp);
    }

    // Returned: task yielded, was preempted, blocked, or exited.
    arch::timer::disarm_apic_timer();
}
