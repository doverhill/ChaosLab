//! Idle loop: each CPU runs this, picking tasks and dispatching them.
//!
//! When no tasks are available, the CPU halts (HLT) until woken by an
//! IPI from another CPU or a timer interrupt. Idle time is tracked per
//! CPU via TSC for utilization metrics.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::{Mutex, Once};

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
    /// Last CR3 (page table) this CPU ran. When idle, the CPU stays in this
    /// address space so the TLB remains warm. Prefer waking a CPU that
    /// already has the right CR3 loaded to avoid TLB flushes.
    last_cr3: AtomicU64,
    /// TSC when this CPU entered idle (0 = not idle).
    idle_since_tsc: AtomicU64,
    /// Accumulated idle TSC ticks for utilization measurement.
    total_idle_tsc: AtomicU64,
    /// Per-CPU local run queue. The owning CPU dequeues without contention.
    /// Remote CPUs may push (unblock a task that last ran here). Tasks that
    /// yield or get preempted go back to their CPU's local queue, avoiding
    /// the global lock on the hot path.
    ///
    /// FIXME: this uses a Mutex<VecDeque> which is simple but means the owning
    /// CPU still takes a lock to dequeue. A lock-free SPMC queue (single
    /// producer = owning CPU, multiple consumers = none, multiple pushers =
    /// remote CPUs) would eliminate all locking on the dequeue path. Chase-Lev
    /// work-stealing deque would be ideal here.
    local_queue: Mutex<VecDeque<TaskId>>,
}

/// Heap-allocated per-CPU idle state. Initialized by `init()`.
static PER_CPU_IDLE: Once<Vec<PerCpuIdleState>> = Once::new();

/// Cached hint: a CPU ID that is known to be idle, or -1 (u64::MAX)
/// if unknown. Updated on begin_idle/end_idle. Avoids scanning the
/// per-CPU array on every wake_idle_cpu call.
const NO_IDLE_CPU: u64 = u64::MAX;
static FIRST_IDLE_CPU: AtomicU64 = AtomicU64::new(NO_IDLE_CPU);

/// Global count of runnable tasks (across all local queues + global queue).
/// Used for load balancing: each CPU compares its local count against
/// total / cpu_count to detect imbalance.
static TOTAL_RUNNABLE: AtomicU64 = AtomicU64::new(0);
/// Number of CPUs (set once during init).
static CPU_COUNT: AtomicU64 = AtomicU64::new(1);

/// Rebalance threshold: only shed tasks if local queue exceeds
/// fair_share + REBALANCE_THRESHOLD.
const REBALANCE_THRESHOLD: usize = 2;
/// How often to check load balance (every Nth scheduling cycle per CPU).
const REBALANCE_INTERVAL: u64 = 8;

/// Initialize per-CPU idle state for the given number of CPU slots.
/// Must be called once after SMP discovery, before any CPU enters the idle loop.
pub fn init(cpu_count: usize) {
    CPU_COUNT.store(cpu_count as u64, Ordering::Relaxed);
    PER_CPU_IDLE.call_once(|| {
        let mut v = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count {
            v.push(PerCpuIdleState {
                idle_rsp_pointer: AtomicU64::new(0),
                current_task_id: AtomicU64::new(0),
                is_idle: AtomicBool::new(false),
                last_cr3: AtomicU64::new(0),
                idle_since_tsc: AtomicU64::new(0),
                total_idle_tsc: AtomicU64::new(0),
                local_queue: Mutex::new(VecDeque::new()),
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

/// Increment the global runnable task count.
/// Call when a task becomes runnable (spawn, unblock, yield re-enqueue).
pub fn increment_runnable() {
    TOTAL_RUNNABLE.fetch_add(1, Ordering::Relaxed);
}

/// Decrement the global runnable task count.
/// Call when a task stops being runnable (dispatched, blocked, exited).
pub fn decrement_runnable() {
    TOTAL_RUNNABLE.fetch_sub(1, Ordering::Relaxed);
}

/// Get the number of tasks in a CPU's local run queue.
pub fn local_queue_len(cpu_id: usize) -> usize {
    per_cpu(cpu_id).local_queue.lock().len()
}

/// Push a task onto a specific CPU's local run queue.
/// Used when unblocking a task that last ran on that CPU (keeps it local).
pub fn push_to_local_queue(cpu_id: usize, task_id: TaskId) {
    per_cpu(cpu_id).local_queue.lock().push_back(task_id);
    increment_runnable();
}

/// Pop from this CPU's local run queue. Returns None if empty.
fn pop_local_queue(cpu_id: usize) -> Option<TaskId> {
    let result = per_cpu(cpu_id).local_queue.lock().pop_front();
    if result.is_some() {
        decrement_runnable();
    }
    result
}

/// How many scheduling cycles between global steals.
/// Every Nth cycle we pull tasks from global into local, ensuring global
/// tasks don't starve when local is busy.
const STEAL_INTERVAL: u64 = 4;

/// Steal up to N tasks from the global run queue into the local queue.
/// Called periodically from the idle loop to prevent global starvation.
/// FIXME: a smarter approach would be to check global queue length and
/// steal proportionally, or use a work-stealing deque where idle CPUs
/// pull from busy CPUs' local queues.
fn steal_from_global_if_needed(cpu_id: usize) {
    // Use a simple counter per CPU to avoid stealing on every iteration
    static STEAL_COUNTERS: spin::Once<Vec<AtomicU64>> = spin::Once::new();
    let counters = STEAL_COUNTERS.call_once(|| {
        let count = PER_CPU_IDLE.get().map_or(1, |v| v.len());
        (0..count).map(|_| AtomicU64::new(0)).collect()
    });

    let cycle = counters[cpu_id].fetch_add(1, Ordering::Relaxed);
    if cycle % STEAL_INTERVAL != 0 { return; }

    // Steal up to 4 tasks from global into local
    let mut state = SCHEDULER.lock();
    let mut local = per_cpu(cpu_id).local_queue.lock();
    for _ in 0..4 {
        if let Some(task_id) = state.run_queue.dequeue() {
            local.push_back(task_id);
        } else {
            break;
        }
    }
}

/// Wake an idle CPU by sending a scheduler IPI. Prefers a CPU whose
/// last_cr3 matches `preferred_cr3` (avoids TLB flush). If no match,
/// wakes any idle CPU. If no CPU is idle, does nothing.
pub fn wake_idle_cpu_with_cr3(preferred_cr3: u64) {
    let per_cpu_vec = match PER_CPU_IDLE.get() {
        Some(v) => v,
        None => return,
    };

    // Fast path: check the cached hint
    let hint = FIRST_IDLE_CPU.load(Ordering::Acquire);
    if hint != NO_IDLE_CPU {
        let hint_cpu = hint as usize;
        if hint_cpu < per_cpu_vec.len() && per_cpu_vec[hint_cpu].is_idle.load(Ordering::Acquire) {
            // If we have a CR3 preference and the hint doesn't match,
            // scan for a better match before falling back to the hint.
            if preferred_cr3 == 0 || per_cpu_vec[hint_cpu].last_cr3.load(Ordering::Relaxed) == preferred_cr3 {
                arch::send_scheduler_ipi(hint_cpu);
                return;
            }
        }
    }

    // Scan: prefer CR3 match, fall back to any idle CPU
    let mut fallback: Option<usize> = None;
    for (id, state) in per_cpu_vec.iter().enumerate() {
        if !state.is_idle.load(Ordering::Acquire) { continue; }
        if preferred_cr3 != 0 && state.last_cr3.load(Ordering::Relaxed) == preferred_cr3 {
            arch::send_scheduler_ipi(id);
            return;
        }
        if fallback.is_none() {
            fallback = Some(id);
        }
    }

    // No CR3 match — wake any idle CPU
    if let Some(id) = fallback {
        arch::send_scheduler_ipi(id);
    }
}

/// Wake any idle CPU (no CR3 preference).
pub fn wake_idle_cpu() {
    // Fast path: check the cached hint
    let hint = FIRST_IDLE_CPU.load(Ordering::Acquire);
    if hint != NO_IDLE_CPU {
        arch::send_scheduler_ipi(hint as usize);
        return;
    }
    // Slow path: scan
    if let Some(per_cpu_vec) = PER_CPU_IDLE.get() {
        for (id, state) in per_cpu_vec.iter().enumerate() {
            if state.is_idle.load(Ordering::Acquire) {
                arch::send_scheduler_ipi(id);
                return;
            }
        }
    }
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
        // Periodic load balancing: shed excess tasks if overloaded,
        // steal from global if local is empty.
        rebalance_if_needed(cpu_id);
        steal_from_global_if_needed(cpu_id);
        let picked = pop_local_queue(cpu_id).or_else(|| {
            let mut state = SCHEDULER.lock();
            state.pick_next()
        });

        if let Some(task_id) = picked {
            end_idle(cpu_id);
            dispatch(cpu_id, task_id, &mut idle_rsp);

            // Re-enqueue if still Running (yield/preempt case).
            // Push to local queue for cache/TLB locality, UNLESS other
            // CPUs are idle and our local queue is already busy — in that
            // case, push to global so idle CPUs can pick up the work.
            //
            // Push to local for locality. The periodic rebalance_if_needed()
            // will shed excess tasks to global if this CPU is overloaded.
            let returned_id = per_cpu(cpu_id).current_task_id.swap(0, Ordering::Release);
            if returned_id != 0 {
                let mut state = SCHEDULER.lock();
                if let Some(task) = state.get(returned_id) {
                    if task.state == TaskState::Running {
                        let local_count = per_cpu(cpu_id).local_queue.lock().len();
                        let has_idle_cpus = FIRST_IDLE_CPU.load(Ordering::Relaxed) != NO_IDLE_CPU;

                        if local_count > 0 && has_idle_cpus {
                            // Shed work to idle CPUs via global queue
                            state.requeue(returned_id);
                            increment_runnable();
                            drop(state);
                            wake_idle_cpu();
                        } else {
                            drop(state);
                            per_cpu(cpu_id).local_queue.lock().push_back(returned_id);
                            increment_runnable();
                        }
                    }
                }
            }
        } else {
            // Nothing to run — check for timed wakeups and halt.
            // Process any expired timers first (might unblock tasks).
            process_expired_timers();

            // Arm the APIC timer for the nearest timer queue deadline so
            // we wake up to unblock sleeping tasks even without an IPI.
            arm_for_timed_wakeup();

            begin_idle(cpu_id);
            arch::halt_until_interrupt();
            end_idle(cpu_id);

            // After waking (IPI or timer), process any expired timers.
            arch::timer::disarm_apic_timer();
            process_expired_timers();
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
// Load balancing
// ---------------------------------------------------------------------------

/// Check if this CPU is overloaded and shed excess tasks to global.
/// Called periodically from the idle loop (every REBALANCE_INTERVAL cycles).
fn rebalance_if_needed(cpu_id: usize) {
    static REBALANCE_COUNTERS: spin::Once<Vec<AtomicU64>> = spin::Once::new();
    let counters = REBALANCE_COUNTERS.call_once(|| {
        let count = PER_CPU_IDLE.get().map_or(1, |v| v.len());
        (0..count).map(|_| AtomicU64::new(0)).collect()
    });

    let cycle = counters[cpu_id].fetch_add(1, Ordering::Relaxed);
    if cycle % REBALANCE_INTERVAL != 0 {
        return;
    }

    let total = TOTAL_RUNNABLE.load(Ordering::Relaxed) as usize;
    let cpus = CPU_COUNT.load(Ordering::Relaxed) as usize;
    if cpus == 0 { return; }

    let fair_share = total / cpus;
    let my_local_len = per_cpu(cpu_id).local_queue.lock().len();

    if my_local_len > fair_share + REBALANCE_THRESHOLD {
        // Shed excess tasks to global queue
        let excess = my_local_len - fair_share;
        let mut local = per_cpu(cpu_id).local_queue.lock();
        let mut state = SCHEDULER.lock();
        for _ in 0..excess {
            if let Some(task_id) = local.pop_back() {
                // Push to global run queue. The task's state should be
                // Runnable (it's on the local queue, not currently running).
                // FIXME: use actual task priority from the task table instead
                // of default 0. Requires locking task_table to read priority,
                // or storing priority alongside TaskId in the local queue.
                state.run_queue.enqueue(task_id, 0);
            }
        }
        drop(state);
        drop(local);
        // Wake an idle CPU if any — it can now steal from global
        wake_idle_cpu();
    }
}

// ---------------------------------------------------------------------------
// Timed wakeups
// ---------------------------------------------------------------------------

/// Process expired timer entries: unblock tasks whose deadline has passed.
fn process_expired_timers() {
    let now = arch::read_tsc();
    let expired = {
        let mut state = SCHEDULER.lock();
        state.timer_queue.pop_expired(now)
    };
    for task_id in expired {
        // Lazy deletion: only unblock if the task is still Blocked.
        // It may have been unblocked by its actual event already.
        super::unblock(task_id);
    }
}

/// Arm the APIC timer for the nearest timer queue deadline.
/// Called before HLT so the CPU wakes up to process timed wakeups.
fn arm_for_timed_wakeup() {
    let deadline = SCHEDULER.lock().timer_queue.peek_deadline();
    if let Some(deadline_ticks) = deadline {
        let now = arch::read_tsc();
        if deadline_ticks > now {
            let delta = deadline_ticks - now;
            // Convert TSC ticks to APIC timer ticks (approximate — both
            // run at ~1 GHz on QEMU so the ratio is close to 1:1).
            let apic_ticks = (delta as u32).max(1);
            arch::timer::arm_apic_timer_ticks(apic_ticks);
        }
        // If deadline already passed, don't arm — process_expired_timers
        // will handle it on the next loop iteration.
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Dispatch a task on this CPU: context-switch to it and return when it
/// yields, gets preempted, blocks, or exits.
fn dispatch(cpu_id: usize, task_id: TaskId, idle_rsp: &mut u64) {
    let (saved_rsp, task_cr3) = {
        let mut state = SCHEDULER.lock();
        let task = match state.get_mut(task_id) {
            Some(t) => t,
            None => return,
        };
        task.state = TaskState::Running;
        task.last_cpu_id = Some(cpu_id);
        (task.saved_rsp, task.cr3)
    };

    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CPU {} dispatching task {} (rsp={:#x})", cpu_id, task_id, saved_rsp);

    // Publish per-CPU state so yield_current and the timer handler can find us
    let per_cpu_state = per_cpu(cpu_id);
    per_cpu_state.idle_rsp_pointer.store(idle_rsp as *mut u64 as u64, Ordering::Release);
    per_cpu_state.current_task_id.store(task_id, Ordering::Release);
    if task_cr3 != 0 {
        per_cpu_state.last_cr3.store(task_cr3, Ordering::Relaxed);
    }

    // Arm the APIC timer for preemption if there are more tasks waiting
    // (on either the local or global queue). Dynamic ticks: only arm when
    // there's contention for this CPU.
    // FIXME: with a lock-free local queue, the next task to run would
    // already be staged here — the timer handler could switch directly
    // to it without going through the idle loop. That eliminates one
    // context switch per preemption.
    // Arm preemption timer if there are other runnable tasks on this CPU's
    // local queue (already popped the one we're dispatching, so len > 0
    // means others are waiting). Also check global in case tasks were
    // just spawned/unblocked there.
    let local_waiting = per_cpu_state.local_queue.lock().len();
    let global_waiting = SCHEDULER.lock().run_queue.count();
    if local_waiting > 0 || global_waiting > 0 {
        arch::timer::arm_apic_timer_milliseconds(5);
    }

    // Save idle RSP and switch to the task.
    unsafe {
        arch::context_switch(idle_rsp as *mut u64, saved_rsp);
    }

    // Returned: task yielded, was preempted, blocked, or exited.
    arch::timer::disarm_apic_timer();
}
