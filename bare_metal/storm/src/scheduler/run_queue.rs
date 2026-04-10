//! Priority-based multi-level run queue.
//!
//! 32 priority levels with a bitmap for O(1) highest-priority lookup.
//! Within a level, tasks are round-robin (FIFO).

use alloc::collections::VecDeque;
use super::task::TaskId;

pub const PRIORITY_LEVELS: usize = 32;
pub const DEFAULT_PRIORITY: i32 = 0;

/// Map a user-facing priority (i32, higher = more important) to a
/// run queue level index (0..31).
fn priority_to_level(priority: i32) -> usize {
    (priority + 16).clamp(0, 31) as usize
}

pub struct RunQueue {
    levels: [VecDeque<TaskId>; PRIORITY_LEVELS],
    /// Bit i is set if levels[i] is non-empty.
    nonempty_bitmap: u32,
    count: usize,
}

impl RunQueue {
    pub const fn new() -> Self {
        const EMPTY: VecDeque<TaskId> = VecDeque::new();
        Self {
            levels: [EMPTY; PRIORITY_LEVELS],
            nonempty_bitmap: 0,
            count: 0,
        }
    }

    /// Add a task to the back of its priority level.
    pub fn enqueue(&mut self, task_id: TaskId, priority: i32) {
        let level = priority_to_level(priority);
        self.levels[level].push_back(task_id);
        self.nonempty_bitmap |= 1 << level;
        self.count += 1;
    }

    /// Remove and return the highest-priority task.
    pub fn dequeue(&mut self) -> Option<TaskId> {
        if self.nonempty_bitmap == 0 {
            return None;
        }
        // Highest priority = highest set bit
        let level = 31 - self.nonempty_bitmap.leading_zeros() as usize;
        let task_id = self.levels[level].pop_front().unwrap();
        if self.levels[level].is_empty() {
            self.nonempty_bitmap &= !(1 << level);
        }
        self.count -= 1;
        Some(task_id)
    }

    /// Number of tasks waiting in the queue.
    pub fn count(&self) -> usize {
        self.count
    }

    /// True if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
