//! Architecture abstraction layer.
//!
//! Each architecture provides implementations of the same interface:
//! page table operations, interrupt/exception handling, syscall mechanism,
//! context switching, timer, and SMP startup.
//!
//! The active architecture is selected at compile time via `target_arch`.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

// Re-export the active architecture's modules so the rest of the kernel
// can use `crate::arch::gdt`, `crate::arch::interrupts`, etc.
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;
