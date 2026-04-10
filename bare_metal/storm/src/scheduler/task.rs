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
    pub cpu_affinity: Option<u32>,
    /// Last CPU this task ran on (for locality).
    pub last_cpu: Option<u32>,
    /// IPC partner task (for scheduling on different CPUs).
    pub ipc_partner: Option<TaskId>,
    /// TSC ticks remaining in current timeslice.
    pub timeslice_remaining_ticks: u64,
}
