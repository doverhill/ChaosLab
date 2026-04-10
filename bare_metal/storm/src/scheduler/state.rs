//! Scheduler state: the task table, run queue, and timer queue.
//!
//! Currently protected by a single Mutex for correctness. The hot path
//! (pick_next → dispatch → yield → requeue) holds the lock briefly.
//!
//! Future optimization: per-CPU local run queues with work stealing.
//! Each CPU has a small local queue (lock-free, only that CPU dequeues).
//! - pick_next: check local first. If empty, lock global and steal a batch.
//! - yield/preempt: push to local (lock-free). Overflow goes to global.
//! - spawn/unblock: push to global (any CPU should pick it up).
//! This means the common case (run → yield → pick up again) never touches
//! the global lock. Additional optimizations:
//! - Make Task.state atomic (check without locking the table)
//! - Task table only locked for spawn/exit (infrequent)

use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU64, Ordering};

use super::task::{Task, TaskId, TaskState};
use super::run_queue::RunQueue;
use super::timer_queue::TimerQueue;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

pub fn allocate_task_id() -> TaskId {
    NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
}

pub struct SchedulerState {
    pub task_table: BTreeMap<TaskId, Box<Task>>,
    pub run_queue: RunQueue,
    pub timer_queue: TimerQueue,
}

impl SchedulerState {
    pub const fn new() -> Self {
        Self {
            task_table: BTreeMap::new(),
            run_queue: RunQueue::new(),
            timer_queue: TimerQueue::new(),
        }
    }

    /// Insert a new task into the task table and enqueue it as Runnable.
    pub fn add_and_enqueue(&mut self, mut task: Task) {
        let task_id = task.task_id;
        let priority = task.priority;
        task.state = TaskState::Runnable;
        self.task_table.insert(task_id, Box::new(task));
        self.run_queue.enqueue(task_id, priority);
    }

    /// Dequeue the highest-priority runnable task and mark it Running.
    /// Returns the TaskId and a reference to the Task.
    pub fn pick_next(&mut self) -> Option<TaskId> {
        let task_id = self.run_queue.dequeue()?;
        if let Some(task) = self.task_table.get_mut(&task_id) {
            task.state = TaskState::Running;
            Some(task_id)
        } else {
            // Task was removed (exited) while on the queue — skip it
            None
        }
    }

    /// Return a running task to the run queue as Runnable.
    pub fn requeue(&mut self, task_id: TaskId) {
        if let Some(task) = self.task_table.get_mut(&task_id) {
            task.state = TaskState::Runnable;
            self.run_queue.enqueue(task_id, task.priority);
        }
    }

    /// Block a running task. Does not enqueue it.
    pub fn block(&mut self, task_id: TaskId, reason: super::task::BlockReason) {
        if let Some(task) = self.task_table.get_mut(&task_id) {
            task.state = TaskState::Blocked;
            task.block_reason = Some(reason);

            // If there's a deadline, insert into the timer queue
            let deadline = match reason {
                super::task::BlockReason::EventWait { timeout_ticks: Some(t) } => Some(t),
                super::task::BlockReason::Sleep { wakeup_ticks } => Some(wakeup_ticks),
                _ => None,
            };
            if let Some(deadline_ticks) = deadline {
                self.timer_queue.insert(task_id, deadline_ticks);
            }
        }
    }

    /// Unblock a task and make it Runnable. Returns true if the task was
    /// actually Blocked (lazy deletion: may have already been unblocked).
    pub fn unblock(&mut self, task_id: TaskId) -> bool {
        if let Some(task) = self.task_table.get_mut(&task_id) {
            if task.state == TaskState::Blocked {
                task.state = TaskState::Runnable;
                task.block_reason = None;
                self.run_queue.enqueue(task_id, task.priority);
                return true;
            }
        }
        false
    }

    /// Mark a task as Exited. Does not remove it from the task table
    /// (cleanup is deferred).
    pub fn exit(&mut self, task_id: TaskId) {
        if let Some(task) = self.task_table.get_mut(&task_id) {
            task.state = TaskState::Exited;
            task.block_reason = None;
        }
    }

    /// Remove an Exited task from the task table entirely.
    pub fn reap(&mut self, task_id: TaskId) -> Option<Box<Task>> {
        if self.task_table.get(&task_id).map_or(false, |t| t.state == TaskState::Exited) {
            self.task_table.remove(&task_id)
        } else {
            None
        }
    }

    /// Get a reference to a task by ID.
    pub fn get(&self, task_id: TaskId) -> Option<&Task> {
        self.task_table.get(&task_id).map(|b| b.as_ref())
    }

    /// Get a mutable reference to a task by ID.
    pub fn get_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.task_table.get_mut(&task_id).map(|b| b.as_mut())
    }
}
