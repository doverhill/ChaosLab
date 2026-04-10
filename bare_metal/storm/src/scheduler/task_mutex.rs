//! Yielding mutex for use from task contexts.
//!
//! Unlike a plain spinlock, this mutex yields the current task's timeslice
//! if the lock is contended, allowing the lock holder to run and release it.
//! This prevents deadlock from preemption and avoids wasting cycles spinning.
//!
//! Use this for any lock that might be held across preemption points (i.e.,
//! any lock acquired by kernel tasks or user syscall handlers). The kernel's
//! scheduler internals should continue using spin::Mutex since they can't
//! yield (the scheduler can't yield to itself).

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use core::sync::atomic::AtomicBool as SchedulerActiveFlag;

/// Maximum spin iterations before yielding.
const SPIN_LIMIT: u32 = 100;

/// Set to true once the scheduler is running. Before this, TaskMutex
/// falls back to pure spinning (yield is not available during boot).
static SCHEDULER_ACTIVE: SchedulerActiveFlag = SchedulerActiveFlag::new(false);

/// Mark the scheduler as active. Called once when the first CPU enters
/// the idle loop.
pub fn set_scheduler_active() {
    SCHEDULER_ACTIVE.store(true, Ordering::Release);
}

fn can_yield() -> bool {
    SCHEDULER_ACTIVE.load(Ordering::Acquire)
}

pub struct TaskMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for TaskMutex<T> {}
unsafe impl<T: Send> Sync for TaskMutex<T> {}

impl<T> TaskMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquire the lock. Spins briefly, then yields if contended.
    pub fn lock(&self) -> TaskMutexGuard<'_, T> {
        loop {
            // Fast path: try to acquire immediately
            if self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                return TaskMutexGuard { mutex: self };
            }

            // Spin briefly — the holder might release very soon
            for _ in 0..SPIN_LIMIT {
                if !self.locked.load(Ordering::Relaxed) {
                    break;
                }
                core::hint::spin_loop();
            }

            // Try once more after spinning
            if self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                return TaskMutexGuard { mutex: self };
            }

            // Still contended — yield our timeslice so the lock holder can run.
            // During boot (before the scheduler is active), just keep spinning.
            if can_yield() {
                super::yield_current();
            }
        }
    }
}

pub struct TaskMutexGuard<'a, T> {
    mutex: &'a TaskMutex<T>,
}

impl<T> Deref for TaskMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for TaskMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for TaskMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
