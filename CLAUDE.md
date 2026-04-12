# ChaosLab

Chaos is an experimental operating system built on the Storm microkernel. The project has two parallel implementations: a bare-metal Rust kernel (`bare_metal/storm/`) targeting x86_64, and a hosted .NET kernel (`Storm/`) that runs on Windows for rapid prototyping and testing of the IPC/service architecture.

## Repository layout

```
bare_metal/          Bare-metal x86_64 implementation (Rust, nightly, no_std)
  storm/             Kernel binary
    src/
      arch/x86_64/   Architecture-specific: GDT, IDT, APIC, syscall, page tables, timer
      arch/aarch64/   Placeholder for ARM port
      arch/riscv64/   Placeholder for RISC-V port
      scheduler/     Preemptive scheduler: task.rs, run_queue.rs, idle.rs, state.rs, timer_queue.rs, task_mutex.rs
      main.rs        Boot sequence, test threads, watchdog, task summary dump
      process.rs     Process/Thread structs, ELF loader (create_from_elf)
      address_space.rs  Per-process L4 page tables, user page allocation
      log.rs         Dual serial+framebuffer logger (async FB via queue + sink task)
      ...
  libraries/
    chaos/           System library — no_std syscall wrappers, heap allocator, main! macro
  applications/
    test/            Test app — calls ProcessEmit syscall, tests heap allocation
  servers/           System servers (empty — to be populated)
  protocols/         IPC protocol definitions (empty — to be populated)
  src/bin/           QEMU launch targets (qemu-uefi.rs, qemu-bios.rs)
Storm/               .NET hosted kernel (Windows, TCP-based IPC emulation)
Library/             Hosted system libraries (library_chaos, library_graphics, library_storage)
Protocol/            IPC protocol definitions and generated code
Documentation/       Design docs including scheduling.md
TODO.md              Tracked work items with checkboxes
```

## Architecture

- **Target:** x86_64, boots via UEFI (bootloader crate v0.11.15)
- **Emulator:** QEMU with configurable CPUs (`SMP=N`), 64 MB RAM
- **Memory model:** L4[0..127] identity-mapped physical, L4[128..255] kernel virtual, L4[256..511] user virtual (high canonical half, sign-extended)
- **Kernel design:** Microkernel with syscalls matching the hosted library (ServiceCreate=100, ProcessEmit=401, MemoryAllocate=800, etc.)
- **Scheduler:** Preemptive, priority-based (32 levels), per-CPU local run queues with global work stealing, APIC timer one-shot (5ms timeslice), HLT idle with IPI wakeup, address-space-aware CPU selection, load-aware rebalancing
- **User processes:** ELF loaded from tar ramdisk, PIE with R_X86_64_RELATIVE relocations, own address space, per-thread kernel+user stacks, heap via MemoryAllocate syscall (talc allocator in library_chaos)
- **IPC:** Not yet implemented for bare metal (see TODO.md)
- **Security:** Capability-based (format: `namespace.operation[:resource]`) — not yet implemented

## Building and running

### Bare-metal kernel

Requires Rust nightly with `x86_64-unknown-none` target, `rust-src`, and `llvm-tools` (configured in `bare_metal/rust-toolchain.toml`).

```sh
cd bare_metal
cargo run --bin qemu-uefi    # UEFI boot in QEMU (default 4 CPUs)
SMP=1 cargo run --bin qemu-uefi  # single CPU mode
```

When testing, always capture full output to a file and then grep/tail it.
Always test both SMP=1 and SMP=4 — single-CPU exposes scheduling bugs that hide behind multi-CPU parallelism.
```sh
cargo run --bin qemu-uefi 2>&1 | tee /tmp/run-output
SMP=1 cargo run --bin qemu-uefi 2>&1 | tee /tmp/run-output-1cpu
grep "userspace\|FAULT\|PANIC\|Thread" /tmp/run-output
# periodic framebuffer screenshots:
SCREENSHOT_INTERVAL_MS=500 cargo run --bin qemu-uefi
```

### Hosted kernel (.NET)

Windows-only. Uses PowerShell build scripts at the repo root:
- `BuildAndRun.ps1` - Full build and run
- `BuildKernelAndProtocols.ps1` - Kernel + protocols
- `BuildApps.ps1` - Applications only
- `Run.ps1` - Run with pre-built binaries

## Key kernel modules (bare_metal/storm/src/)

| File | Purpose |
|------|---------|
| `main.rs` | Boot sequence, test threads, watchdog with task summary dump |
| `arch/x86_64/mod.rs` | Arch abstraction: init_cpu, init_interrupts, context_switch, enter_usermode, etc. |
| `arch/x86_64/gdt.rs` | GDT with kernel+user segments, TSS with IST stacks |
| `arch/x86_64/interrupts.rs` | IDT, naked APIC timer handler for preemption, IPI handler |
| `arch/x86_64/syscall.rs` | SYSCALL/SYSRET MSRs, naked entry, per-CPU state, ProcessEmit/MemoryAllocate/MemoryFree handlers |
| `arch/x86_64/timer.rs` | TSC + APIC timer calibration, one-shot arm/disarm, PM timer delays |
| `arch/x86_64/apic.rs` | ACPI MADT parsing, BSP LAPIC init, AP startup via INIT-SIPI-SIPI |
| `arch/x86_64/page_tables.rs` | 4-level page tables, canonical address helpers, VirtualAddressRange |
| `arch/x86_64/memory_setup.rs` | Identity mapping, bootloader decoupling |
| `scheduler/mod.rs` | Public API: spawn_kernel, yield_current, block_current, unblock, exit_current |
| `scheduler/task.rs` | Task (TCB), TaskState, TaskKind, CpuUsageBuckets metrics |
| `scheduler/run_queue.rs` | 32-level priority queue with bitmap for O(1) dequeue |
| `scheduler/idle.rs` | Per-CPU idle loop, local queues, work stealing, rebalancing, HLT, IPI wake |
| `scheduler/timer_queue.rs` | BinaryHeap min-heap for timed wakeups |
| `scheduler/task_mutex.rs` | Yielding mutex for task contexts (spin briefly, then yield) |
| `process.rs` | Process/Thread structs, ELF loader with relocation support |
| `address_space.rs` | Per-process L4 page table, user page allocation with tracking |
| `kernel_memory.rs` | talc heap allocator with VirtualMemorySource |
| `physical_memory.rs` | Physical frame allocator (linked-list) |
| `virtual_memory.rs` | Kernel virtual page allocator |
| `log.rs` | Dual serial+framebuffer, async FB via queue + sink task |
| `framebuffer.rs` | Pixel-level text rendering with noto-sans-mono-bitmap |

## Collaboration style

- **Push back on bad ideas.** Claude should not blindly accept the user's input. If a suggestion has a flaw or there's a better approach, Claude should say so and explain why. The user values honest technical feedback over agreement.
- **Questions are genuine.** When the user ends something with a question mark, they are genuinely asking for Claude's honest opinion or analysis. Give a real assessment, not a diplomatic non-answer.
- **Don't jump to conclusions on bugs.** Verify assumptions with logging or raw serial. Check simplest explanations first. When a fix doesn't work, step back and question the diagnosis instead of stacking more fixes.
- **Always test SMP=1 and SMP=4.** Single-CPU exposes scheduling bugs (priority starvation, missing yields, local queue issues) that are invisible on multi-CPU.
- **No fixed MAX constants.** Use Vec/heap allocation sized from runtime values, not MAX_CPUS arrays.
- **Leave FIXME/TODO comments** on code that could be improved later.
- **Prefer fine-grained locking** (per-object, per-CPU) unless it leads to deadlocks. Use lock-free structures where possible. Consider per-CPU local state with work stealing over global locks.
- **Keep debug logging in code**, controlled via `SERIAL_LOG_LEVEL` in log.rs. Don't remove debug log_println! calls — set the level to Information for normal runs, Debug when troubleshooting.
- **Save progress to CLAUDE.md/TODO.md** during long sessions so context isn't lost between conversations. Update these files before ending a session.

## Coding conventions

- **Rust edition:** 2021
- **Line width:** 200 characters (`rustfmt.toml`: `max_width = 200`)
- **Kernel code:** `#![no_std]` with `extern crate alloc`; nightly features (`abi_x86_interrupt`)
- **Naming:** snake_case for functions/modules, PascalCase for types. Prefer full words over abbreviations in identifiers (e.g. `virtual_to_physical` not `virt_to_phys`, `physical_address` not `phys_addr`, `allocate_frame` not `alloc_frame`, `l4_index` not `l4_idx`). No u32 on a 64-bit-only kernel — use u64 or usize.
- **Logging:** Use `log_println!(SubSystem, LogLevel, ...)` macro. Serial is synchronous, framebuffer is async via log sink task. Control verbosity via `SERIAL_LOG_LEVEL` constant in log.rs.
- **Error types:** `StormError` enum
- **Unsafe:** Used where required for hardware access; comment explaining why

## Syscall ABI

```
rax = syscall number
rdi = arg1, rsi = arg2, rdx = arg3, r10 = arg4
Returns: rax = result
```

Syscall numbers (matching Library/Chaos/src/syscalls.rs):
- 100: ServiceCreate, 101: ServiceSubscribe
- 200: ChannelSignal
- 300: EventWait
- 400: ProcessCreate, 401: ProcessEmit (also used as exit with emit_type=0)
- 500: TimerCreate
- 700: ThreadCreate, 701: ThreadDestroy
- 800: MemoryAllocate, 801: MemoryFree, 802: MemoryMap, 803: MemoryUnmap
- 1000: HandleDestroy

Currently implemented: 401 (ProcessEmit + exit hack), 800 (MemoryAllocate), 801 (MemoryFree).

## Current state

The bare-metal Storm kernel boots on 1-4 CPUs with preemptive scheduling, runs kernel threads and user-space ELF processes from a tar ramdisk. User apps link against library_chaos which provides the main! macro, panic handler, heap allocator (talc + MemoryAllocate syscall), and syscall wrappers.

The scheduler has priority-based run queues, per-CPU local queues with work stealing, APIC timer preemption, HLT idle with IPI wakeup, address-space-aware CPU selection, and load-aware rebalancing. Task metrics track CPU time and recent CPU%.

**Next steps** (see TODO.md for full list):
1. Fix user thread tracking — make user threads proper Tasks instead of piggybacking on launcher kernel tasks
2. Implement IPC syscalls (ChannelSignal, EventWait, ServiceCreate, ServiceSubscribe)
3. Implement proper ProcessDestroy/exit

The hosted .NET Storm kernel has a more complete implementation of the service/channel/event architecture and is used for developing the userspace stack (servers, protocols, applications).
