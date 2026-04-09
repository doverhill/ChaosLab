# ChaosLab

Chaos is an experimental operating system built on the Storm microkernel. The project has two parallel implementations: a bare-metal Rust kernel (`bare_metal/storm/`) targeting x86_64, and a hosted .NET kernel (`Storm/`) that runs on Windows for rapid prototyping and testing of the IPC/service architecture.

## Repository layout

```
bare_metal/          Bare-metal x86_64 implementation (Rust, nightly, no_std)
  storm/             Kernel binary (boot, GDT, interrupts, memory, ACPI, scheduler)
  libraries/
    chaos/           System library — no_std syscall wrappers for user apps
  applications/
    test/            Test app — calls ProcessEmit syscall to log from userspace
  servers/           System servers (empty — to be populated)
  protocols/         IPC protocol definitions (empty — to be populated)
  src/bin/           QEMU launch targets (qemu-uefi.rs, qemu-bios.rs)
Storm/               .NET hosted kernel (Windows, TCP-based IPC emulation)
Library/             Hosted system libraries (library_chaos, library_graphics, library_storage)
Protocol/            IPC protocol definitions and generated code (Console, Data, Filesystem, Storage, Tornado)
Server/              Hosted system servers (Root, Tornado window manager)
HostServer/          Bridge servers for hosted mode (Console via SDL2, Filesystem)
Application/         Hosted user applications (Cluido file browser)
Experiments/         Prototypes and test code
IDLCompiler/         C# tool that generates Rust protocol code from .idl files
Documentation/       Design docs (kernel API, IPC channel layout)
build/               Pre-built .NET executables
startup.list         Startup sequence for hosted mode
```

## Architecture

- **Target:** x86_64, boots via UEFI or legacy BIOS (bootloader crate)
- **Emulator:** QEMU with 4 CPUs, 64 MB RAM
- **Memory model:** Bottom half (bit 47 = 0) is identity-mapped physical memory; upper half is per-process virtual address space
- **Kernel design:** Microkernel with ~10 syscalls (ServiceCreate, ServiceConnect, ChannelMessage, EventWait, ProcessCreate, etc.)
- **IPC:** Full-duplex shared-memory channels with message queues; protocols defined via IDL
- **Security:** Capability-based (format: `namespace.operation[:resource]`)

## Building and running

### Bare-metal kernel

Requires Rust nightly with `x86_64-unknown-none` target, `rust-src`, and `llvm-tools` (configured in `bare_metal/rust-toolchain.toml`).

```sh
cd bare_metal
cargo run --bin qemu-uefi    # UEFI boot in QEMU
cargo run --bin qemu-bios    # Legacy BIOS boot in QEMU
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
| `main.rs` | Entry point, boot sequence orchestration |
| `gdt.rs` | GDT + TSS setup with dedicated IST stacks for double/page faults |
| `interrupts.rs` | IDT exception handlers |
| `physical.rs` | Physical frame allocator (linked-list, magic: `0xC0CA_C07A_DEAD_BEAF`) |
| `kernel_memory.rs` | Slab allocator (9 size classes: 4-1024 bytes), implements GlobalAlloc |
| `apic.rs` | ACPI/MADT parsing, processor discovery |
| `log.rs` | Colored serial output (UART 0x3F8) with subsystem tags and log levels |
| `process.rs` | Process management (placeholder) |
| `syscall.rs` | Syscall handler (placeholder) |

## Coding conventions

- **Rust edition:** 2021
- **Line width:** 200 characters (`rustfmt.toml`: `max_width = 200`)
- **Kernel code:** `#![no_std]` with `extern crate alloc`; nightly features (`abi_x86_interrupt`)
- **Naming:** snake_case for functions/modules, PascalCase for types. Prefer full words over abbreviations in identifiers (e.g. `virtual_to_physical` not `virt_to_phys`, `physical_address` not `phys_addr`, `allocate_frame` not `alloc_frame`, `l4_index` not `l4_idx`)
- **Logging:** Use `log_println!(SubSystem, LogLevel, ...)` macro, not raw serial writes
- **Error types:** `StormError` enum, `ErrorOr<T>` result type
- **Unsafe:** Used where required for hardware access; comment explaining why

## IPC protocol system

Protocols are defined in `.idl` files and compiled to Rust by the IDL compiler. Each protocol produces:
- Message ID constants
- Serialization/deserialization for channel objects
- Client and server wrapper types

Current protocols: Console (text/drawing), Data (structured data), Filesystem, Storage, Tornado (GUI framework).

## Current state

The bare-metal Storm kernel boots on 4 CPUs, runs a cooperative scheduler with kernel threads, and has LAPIC interrupt handlers. Current work: implementing syscalls and loading a user-space ELF app from the ramdisk.

The hosted .NET Storm kernel has a more complete implementation of the service/channel/event architecture and is used for developing the userspace stack (servers, protocols, applications).
