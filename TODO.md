# TODO

## Scheduler
- [x] Phase 1: Unified Task model + priority run queue
- [x] Phase 2: Preemptive scheduling via APIC timer
- [x] Phase 3: HLT idle + IPI wakeup + idle metrics
- [x] Phase 4: Address-space-aware idle + timed wakeups
- [x] Phase 5: IPC affinity — likely unnecessary: address-space-aware idle selection naturally separates IPC partners onto different CPUs
- [x] Phase 6: Per-CPU local run queues with work stealing
- [x] Load-aware rebalancing: overloaded CPUs shed tasks to global based on fair_share
- [x] Per-task metrics: total CPU ticks, 5-second CPU% sliding window, task summary dump
- [ ] Proper work-stealing: idle CPUs steal from busy CPUs' local queues (not just global)
- [ ] Lock-free Chase-Lev work-stealing deque for local queues (currently Mutex<VecDeque>)
- [ ] Track task priority in local queue entries (rebalance currently enqueues at priority 0)

## Scheduler — known issues
- [ ] **User threads not tracked as Tasks**: user processes piggyback on launcher kernel tasks. The launcher calls `enter_usermode` (iretq) and never returns to the idle loop. Process exit calls `run_on_cpu` directly (creates a new idle loop), abandoning the launcher task. Result: user processes show "running" / 0ms in task summary, their CPU time is unaccounted. Fix: implement `spawn_user` that creates `TaskKind::UserThread` tasks with proper kernel stack setup so `context_switch` returns to a trampoline that does iretq. Process exit should call `exit_current()` which marks the task Exited and returns to idle.
- [ ] **Process exit hack**: syscall 401 with emit_type=0 does `run_on_cpu(cpu_id)` which abandons the current kernel stack and creates a fresh idle loop. This leaks the task's kernel stack and leaves the Task in the task_table forever as "running". Needs proper `exit_current()` that marks Exited, frees resources, and context-switches back to idle.

## IPC
- [ ] ChannelSignal, EventWait, ServiceCreate, ServiceSubscribe syscalls
- [ ] Shared-memory channels between processes
- [ ] IDL protocol code generation for bare metal

## Process management
- [ ] Proper ProcessDestroy syscall (see scheduler known issues above)
- [ ] ThreadCreate/ThreadDestroy syscalls
- [ ] Process capabilities and trust chain
- [ ] Process table (global BTreeMap<ProcessId, Box<Process>>) — currently processes are only held in PENDING_PROCESSES during boot

## Memory
- [x] MemoryAllocate/MemoryFree syscalls
- [x] Per-address-space user page count tracking
- [ ] MemoryMap/MemoryUnmap syscalls (shared memory between processes)
- [ ] User pointer validation in syscall handlers (currently trusts user pointers)

## Logging
- [x] Async framebuffer log via queue + sink task (avoids preemption deadlock)
- [x] Interrupt-disable around serial write + queue push in _print
- [ ] Known QEMU quirk: VGA dirty page tracking causes occasional blank rows in framebuffer output (not a kernel bug — data is correct, display refresh artifact)

## Architecture
- [ ] AArch64 port (placeholder dirs exist at bare_metal/storm/src/arch/aarch64/)
- [ ] RISC-V port (placeholder dirs exist at bare_metal/storm/src/arch/riscv64/)

## Build / tooling
- [x] Strip debug info from ramdisk ELFs (1 MiB → 84 KiB per app)
- [x] Tar-based ramdisk with multiple apps
- [x] Periodic screenshots via SCREENSHOT_INTERVAL_MS env var
- [x] SMP count configurable via SMP env var
- [ ] Release mode builds for apps
