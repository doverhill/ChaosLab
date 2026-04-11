# TODO

## Scheduler
- [x] Phase 1: Unified Task model + priority run queue
- [x] Phase 2: Preemptive scheduling via APIC timer
- [x] Phase 3: HLT idle + IPI wakeup + idle metrics
- [ ] Phase 4: Address-space-aware idle + timed wakeups
- [ ] Phase 5: IPC affinity (depends on IPC syscalls)
- [ ] Phase 6: Per-CPU local run queues with work stealing

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
