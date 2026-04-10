# Storm Scheduler Design

## Overview

The Storm scheduler manages all executable work — kernel threads and user
threads — as a single unified entity called a **Task**. It implements
priority-based preemptive scheduling with IPC-aware CPU assignment and
efficient idle behavior.

## Task Model

Every schedulable entity is a `Task` (the Thread Control Block / TCB):

```
Task
├── task_id: u64            (globally unique)
├── state: TaskState        (Running, Runnable, Blocked, Exited)
├── kind: TaskKind          (Kernel or UserThread { process_id, local_thread_id })
├── priority: i32           (higher = more important, default 0)
├── saved_rsp: u64          (kernel RSP pointing at saved register frame)
├── kernel_stack_base/top   (each task owns a kernel stack)
├── cr3: u64                (page table physical address, 0 for kernel tasks)
├── block_reason: Option    (EventWait/Sleep/ChannelSignal + optional timeout)
├── last_cpu: Option<u32>   (for locality)
├── ipc_partner: Option     (for IPC affinity)
└── timeslice_remaining     (TSC ticks left in current quantum)
```

A user **process** is not itself runnable — its **threads** are. `TaskKind`
reflects this:

```rust
enum TaskKind {
    Kernel,                                     // standalone kernel thread
    UserThread { process_id: u64, local_thread_id: u64 },  // thread within a process
}
```

A `UserThread` task references its parent process (which owns the address
space, handles, capabilities). Multiple tasks can share the same process_id
— these are threads within the same process. The `cr3` is looked up from
the process table via `process_id`.

The context switch mechanism is identical for both kinds: `saved_rsp` always
points to a kernel stack with callee-saved registers. For user thread tasks,
the kernel stack also contains the full user register frame (pushed by the
syscall entry or timer interrupt).

## State Transitions

```
              spawn()
                │
                ▼
  ┌──────── RUNNABLE ◄──────────┐
  │             │                │
  │        pick_next()      wake() / unblock()
  │             │                │
  │             ▼                │
  │         RUNNING ──────── BLOCKED
  │             │    block()
  │             │
  │       yield_current()
  │       preempt() (timer)
  │             │
  └─────────────┘
                │
          exit_current()
                │
                ▼
            EXITED
```

## Run Queue

Priority-based multi-level queue with 32 levels and a bitmap for O(1)
highest-priority lookup:

```
Level 31 (highest): [task_3, task_7]     bitmap: ...1000...
Level 30:           []
...
Level 16 (default): [task_1, task_5]     bitmap: ...0001...
...
Level 0 (lowest):   [task_9]             bitmap: 0000...001
```

`dequeue()` finds the highest set bit in the bitmap, pops from that level's
VecDeque. Within a level, tasks run round-robin (FIFO).

The run queue stores `TaskId` values, not owned `Task` objects. All tasks
live in a global `BTreeMap<TaskId, Box<Task>>` (the task table). This avoids
moving large structs and allows multiple structures (run queue, timer queue,
per-CPU state) to reference the same task.

## Scheduler State

### Initial implementation

A single `Mutex<SchedulerState>` contains the task table, run queue, and
timer queue. Correct and simple for the initial implementation.

### Target design: per-CPU local state with work stealing

Each CPU has a **local state** (lock-free, only that CPU dequeues) containing
a small run queue and a set of blocked tasks. A global queue holds tasks
that haven't been claimed by any CPU.

- **pick_next**: check local run queue first (lock-free). If empty, lock
  the global queue and steal a batch (2-4 tasks).
- **yield/preempt**: push to local run queue (lock-free). If the local
  queue overflows or other CPUs are starved, push to global.
- **block (EventWait/sleep)**: task stays in the CPU's local blocked set.
  It is NOT returned to global. When the unblock event arrives, the task
  goes back to the same CPU's local run queue.
- **spawn/unblock-from-other-CPU**: push to global queue (any CPU can pick it up,
  respecting affinity hints).

This means a well-behaved service thread that does `EventWait → handle →
signal → EventWait` stays in one CPU's local state indefinitely: zero lock
contention, warm caches, warm TLB. The task only leaves local state if:
- The CPU has too many local tasks (rebalance to global)
- IPC affinity hint suggests a different CPU
- The CPU is going idle and donates its local tasks back to global

## Per-CPU State

Each CPU has a `PerCpuState` struct (heap-allocated, indexed by LAPIC ID):

```
PerCpuState
├── kernel_rsp          (for syscall entry asm — offset 0)
├── scratch_rsp         (for syscall entry asm — offset 8)
├── current_process_ptr (for syscall handler  — offset 16)
├── current_thread_ptr  (for syscall handler  — offset 24)
├── idle_rsp            (saved RSP of the idle loop)
├── current_task_id     (what's running now)
├── last_cr3            (address space of last user task — for TLB affinity)
├── idle_since_tsc      (when this CPU entered idle, 0 if busy)
├── total_idle_tsc      (accumulated idle time)
├── timer_armed         (is APIC timer set?)
└── timer_deadline_tsc  (when it will fire)
```

The first 32 bytes have fixed layout for the naked syscall entry assembly.

## Preemption

Preemption uses the APIC timer in **one-shot mode** with **dynamic ticks**:

- When dispatching a task, if the run queue has more waiting tasks, arm the
  APIC timer for the task's timeslice (2-10ms, proportional to priority).
- If the run queue is empty, do not arm — no need to preempt.
- The timer is re-evaluated each time a task is dispatched.

When the timer fires:
1. Check the interrupt frame's CS. If Ring 0 (kernel code), do not preempt
   (kernel code may hold spinlocks). Just EOI and return.
2. If Ring 3 (user code): save full register set on the kernel stack, store
   `saved_rsp` in the task's TCB, set state to Runnable, enqueue, then
   `switch_to_idle_rsp` to return to the idle loop.

The idle loop picks the next task normally. This reuses the same dispatch
path for both cooperative yield and preemption.

### Why only preempt user mode?

Preempting kernel code that holds a spinlock causes deadlock if the next task
tries to acquire the same lock. The proper solution is a per-CPU preemption
counter (increment around lock acquisitions, only preempt if counter == 0).
For initial implementation, checking Ring 3 is simpler and sufficient — user
processes are the primary preemption target. Kernel threads that hold locks
for excessive time are a kernel bug, not a scheduling problem.

## Idle Behavior

When no runnable tasks exist, the CPU enters idle:

```
loop {
    task = pick_next()
    if task:
        end_idle()
        arm_preemption_timer_if_needed(task)
        dispatch(task)
        disarm_timer()
    else:
        begin_idle()               // record TSC, set idle bitmap
        arm_timed_wakeup()         // APIC timer for nearest deadline
        enable_interrupts + hlt    // sleep until IPI or timer
        disable_interrupts
        process_expired_timers()
}
```

### Idle-in-last-address-space

When entering idle, the CPU does NOT switch CR3 back to the kernel's page
tables. It stays in the last user task's address space. This means the TLB
entries for that address space remain warm.

When a task becomes runnable, `find_best_idle_cpu` checks:
1. An idle CPU whose `last_cr3` matches the task's `cr3` → no TLB flush
2. Any other idle CPU → TLB flush (but still better than preempting a busy CPU)
3. No idle CPU → the task waits on the run queue for preemption

### Timed wakeups

A **timer queue** (min-heap ordered by `wakeup_tsc`) tracks blocked tasks
with timeouts. When entering idle:

1. Peek the nearest deadline from the timer queue.
2. If it exists and is in the future, arm the APIC timer for that deadline.
3. HLT. The CPU wakes from either the timer or an IPI (whichever comes first).
4. After waking, process all expired timer entries: unblock tasks whose
   deadline has passed.

Lazy deletion: when a timer entry fires but the task is no longer Blocked
(it was unblocked by its actual event before the timeout), the entry is
silently discarded.

## IPI Wakeup

When a task becomes runnable (spawned, unblocked, yielded with others waiting),
the scheduler may need to wake an idle CPU:

1. Check `IDLE_CPU_BITMAP` (AtomicU32, one bit per CPU).
2. Find best idle CPU (prefer CR3 match, avoid IPC partner's CPU).
3. Send a fixed IPI to that CPU using vector 0xFC.
4. The IPI handler just does EOI — the CPU wakes from HLT and re-enters
   the idle loop, which picks up the newly runnable task.

## IPC Affinity

When `ChannelSignal(channel_handle)` is processed:
1. The kernel identifies the signaller and the target task.
2. Both tasks record each other as `ipc_partner`.
3. When unblocking the target, `find_best_idle_cpu` avoids the signaller's
   CPU, so the two tasks run concurrently on different CPUs.

This is a lightweight hint, not a hard constraint. If the only idle CPU is
the signaller's, it's used anyway. The partner is cleared on the next
block/yield to avoid stale affinity.

## Timeslice Computation

Higher-priority tasks get longer timeslices:

```
level = (priority + 16).clamp(0, 31)
timeslice_ms = 2 + level / 4          // 2ms (level 0) to 10ms (level 31)
timeslice_tsc = timeslice_ms * tsc_frequency / 1000
```

This means high-priority tasks, once scheduled, run longer before preemption.
Low-priority tasks get shorter quanta but still run if no higher-priority
tasks are waiting.

## CPU Utilization Metrics

Each CPU tracks idle time via TSC:

```
begin_idle: idle_since_tsc = rdtsc()
end_idle:   total_idle_tsc += rdtsc() - idle_since_tsc

utilization = 100 - (total_idle_tsc * 100 / total_tsc_since_boot)
```

This can be queried via a future diagnostic syscall or logged periodically.

## File Organization

```
storm/src/
    scheduler/
        mod.rs           — Public API: spawn, yield, block, unblock, exit
        task.rs          — Task, TaskId, TaskState, TaskKind, BlockReason
        run_queue.rs     — Priority multi-level queue
        timer_queue.rs   — Timed wakeup min-heap
        idle.rs          — Idle loop, metrics, HLT
        state.rs         — SchedulerState (task table + run queue + timer queue)
    per_cpu.rs           — Unified PerCpuState
    arch/x86_64/
        context.rs       — context_switch, resume_from_preemption (naked asm)
        ipi.rs           — IPI sending
        timer.rs         — APIC timer arm/disarm, TSC calibration
        interrupts.rs    — Naked timer handler, IPI handler
```
