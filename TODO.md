# TODO

## Scheduler
- [x] Phase 1: Unified Task model + priority run queue
- [x] Phase 2: Preemptive scheduling via APIC timer
- [x] Phase 3: HLT idle + IPI wakeup + idle metrics
- [x] Phase 4: Address-space-aware idle + timed wakeups
- [x] Phase 5: IPC affinity — likely unnecessary: address-space-aware idle selection naturally separates IPC partners onto different CPUs (each CPU idles in its last CR3, so unblocked tasks return to "their" CPU)
- [x] Phase 6: Per-CPU local run queues with work stealing
- [ ] Proper work-stealing: idle CPUs steal from busy CPUs' local queues (not just global)
- [ ] Lock-free Chase-Lev work-stealing deque for local queues
- [ ] Rebalancing when all CPUs are busy but load is uneven

## IPC
- [ ] ChannelSignal, EventWait, ServiceCreate, ServiceSubscribe syscalls
- [ ] Shared-memory channels between processes
- [ ] IDL protocol code generation for bare metal

## Process management
- [ ] Proper ProcessDestroy syscall (currently exit is a hack via ProcessEmit type=0)
- [ ] ThreadCreate/ThreadDestroy syscalls
- [ ] Process capabilities and trust chain

## Memory
- [x] MemoryAllocate/MemoryFree syscalls
- [ ] MemoryMap/MemoryUnmap syscalls (shared memory between processes)
- [ ] User pointer validation in syscall handlers

## Architecture
- [ ] AArch64 port (placeholder dirs exist)
- [ ] RISC-V port (placeholder dirs exist)

## Build / tooling
- [x] Strip debug info from ramdisk ELFs
- [x] Tar-based ramdisk with multiple apps
- [ ] Release mode builds for apps
