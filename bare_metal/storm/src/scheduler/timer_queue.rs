//! Timer queue for timed wakeups (EventWait with timeout, sleep).
//!
//! A min-heap ordered by deadline. Uses lazy deletion — when an entry
//! fires but the task is no longer Blocked, it is silently discarded.

use alloc::collections::BinaryHeap;
use core::cmp::{Ordering, Reverse};
use super::task::TaskId;

#[derive(Debug, Clone, Copy)]
pub struct TimerEntry {
    pub deadline_ticks: u64,
    pub task_id: TaskId,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.deadline_ticks == other.deadline_ticks
    }
}

impl Eq for TimerEntry {}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deadline_ticks.cmp(&other.deadline_ticks)
    }
}

pub struct TimerQueue {
    /// BinaryHeap is a max-heap, so we wrap in Reverse for min-heap behavior.
    heap: BinaryHeap<Reverse<TimerEntry>>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Insert a timed wakeup.
    pub fn insert(&mut self, task_id: TaskId, deadline_ticks: u64) {
        self.heap.push(Reverse(TimerEntry { deadline_ticks, task_id }));
    }

    /// Peek at the nearest deadline without removing it.
    pub fn peek_deadline(&self) -> Option<u64> {
        self.heap.peek().map(|entry| entry.0.deadline_ticks)
    }

    /// Pop all entries whose deadline has passed (deadline <= now).
    /// Returns the task IDs. Caller is responsible for checking whether
    /// each task is still Blocked (lazy deletion).
    pub fn pop_expired(&mut self, now_ticks: u64) -> alloc::vec::Vec<TaskId> {
        let mut expired = alloc::vec::Vec::new();
        while let Some(entry) = self.heap.peek() {
            if entry.0.deadline_ticks > now_ticks {
                break;
            }
            expired.push(self.heap.pop().unwrap().0.task_id);
        }
        expired
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
