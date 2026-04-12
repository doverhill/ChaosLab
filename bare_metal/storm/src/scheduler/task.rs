//! Task: the unified schedulable entity (Thread Control Block).
//!
//! Every schedulable unit — kernel thread or user thread — is a Task.
//! A user process is not itself runnable; its threads are.

/// Globally unique task identifier.
pub type TaskId = u64;

/// Process identifier (for user threads).
pub type ProcessId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Currently executing on a CPU.
    Running,
    /// Ready to run, sitting on the run queue.
    Runnable,
    /// Waiting for an event, timer, or IPC signal.
    Blocked,
    /// Finished execution, awaiting cleanup.
    Exited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    /// A standalone kernel thread (no parent process, runs in kernel address space).
    Kernel,
    /// A thread within a user process.
    UserThread {
        process_id: ProcessId,
        local_thread_id: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Waiting for an event, optionally with a timeout.
    EventWait { timeout_ticks: Option<u64> },
    /// Sleeping until a specific TSC value.
    Sleep { wakeup_ticks: u64 },
    /// Waiting for a channel signal.
    ChannelSignal { channel_handle: u64 },
}

/// The Thread Control Block — everything the scheduler needs to manage
/// a schedulable entity.
pub struct Task {
    pub task_id: TaskId,
    pub state: TaskState,
    pub kind: TaskKind,
    pub priority: i32,

    // Context save area
    /// Kernel RSP pointing at the saved register frame. For kernel tasks,
    /// this is the only RSP. For user tasks, the kernel stack also contains
    /// the full user register frame below this point.
    pub saved_rsp: u64,
    pub kernel_stack_base: u64,
    pub kernel_stack_top: u64,

    // User-mode specifics (only meaningful for UserThread tasks)
    pub user_stack_top: u64,
    pub entry_point: u64,
    /// Page table physical address. 0 for kernel tasks (uses kernel CR3).
    pub cr3: u64,

    // Blocking
    pub block_reason: Option<BlockReason>,

    // Scheduling metadata
    /// Preferred CPU (LAPIC ID). None = any CPU.
    pub cpu_affinity: Option<usize>,
    /// Last CPU this task ran on (for locality).
    pub last_cpu_id: Option<usize>,
    /// IPC partner task (for scheduling on different CPUs).
    pub ipc_partner: Option<TaskId>,
    /// TSC ticks remaining in current timeslice.
    pub timeslice_remaining_ticks: u64,

    // Metrics
    /// Total TSC ticks this task has spent running on a CPU (cumulative).
    pub total_running_ticks: u64,
    /// TSC when this task was last dispatched (0 if not currently running).
    pub dispatch_tsc: u64,
    /// Ring buffer of running ticks per second for the last N seconds.
    /// Used to compute recent CPU% as an average over the window.
    pub cpu_usage_buckets: CpuUsageBuckets,
}

/// Ring buffer tracking running ticks per second over a sliding window.
/// Each bucket covers one second. The current bucket accumulates ticks
/// as the task runs; when the second rolls over, we advance to the next
/// bucket and zero it.
pub const CPU_USAGE_WINDOW_SECONDS: usize = 5;

pub struct CpuUsageBuckets {
    /// Ticks accumulated in each bucket (one per second).
    pub buckets: [u64; CPU_USAGE_WINDOW_SECONDS],
    /// Index of the current (active) bucket.
    pub current_index: usize,
    /// TSC value at the start of the current second.
    pub current_second_start_tsc: u64,
}

impl CpuUsageBuckets {
    pub const fn new() -> Self {
        Self {
            buckets: [0; CPU_USAGE_WINDOW_SECONDS],
            current_index: 0,
            current_second_start_tsc: 0,
        }
    }

    /// Add running ticks and advance buckets if a second boundary was crossed.
    /// `tsc_frequency` is ticks per second (for detecting second boundaries).
    pub fn record_ticks(&mut self, ticks: u64, now_tsc: u64, tsc_frequency: u64) {
        if tsc_frequency == 0 { return; }

        // Advance buckets if we've crossed second boundaries
        if self.current_second_start_tsc == 0 {
            self.current_second_start_tsc = now_tsc;
        }
        let elapsed_since_bucket_start = now_tsc.saturating_sub(self.current_second_start_tsc);
        let seconds_elapsed = elapsed_since_bucket_start / tsc_frequency;
        if seconds_elapsed > 0 {
            // Advance by however many seconds passed (clearing skipped buckets)
            let advance = (seconds_elapsed as usize).min(CPU_USAGE_WINDOW_SECONDS);
            for _ in 0..advance {
                self.current_index = (self.current_index + 1) % CPU_USAGE_WINDOW_SECONDS;
                self.buckets[self.current_index] = 0;
            }
            self.current_second_start_tsc = now_tsc;
        }

        self.buckets[self.current_index] += ticks;
    }

    /// Average ticks per second over the window. Divide by tsc_frequency
    /// to get a 0.0-1.0 CPU fraction, or by tsc_frequency/100 for percent.
    pub fn average_ticks_per_second(&self) -> u64 {
        let sum: u64 = self.buckets.iter().sum();
        sum / CPU_USAGE_WINDOW_SECONDS as u64
    }
}
