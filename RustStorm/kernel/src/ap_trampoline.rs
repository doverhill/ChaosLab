//! Application Processor (AP) trampoline.
//!
//! APs start in 16-bit real mode at the SIPI vector address. This module
//! provides the trampoline binary (assembled from `src/asm/ap_trampoline.asm`
//! by NASM via `build.rs`) and functions to install and patch it.
//!
//! The trampoline is placed at physical address 0x8000 (SIPI vector = 0x08).
//! The BSP patches data fields in the trampoline page before sending each SIPI.
//!
//! ## Mailbox layout (offsets from 0x8000)
//!
//! | Offset | Size | Field |
//! |--------|------|-------|
//! | 0x08   | u64  | ready flag (AP sets to 1 when done with trampoline) |
//! | 0x10   | u64  | CR3 (page table physical address) |
//! | 0x18   | u64  | stack top pointer |
//! | 0x20   | u64  | Rust entry point address |

/// Physical address where the trampoline is placed.
pub const TRAMPOLINE_ADDRESS: u64 = 0x8000;

/// SIPI vector byte (TRAMPOLINE_ADDRESS / 0x1000).
pub const SIPI_VECTOR: u8 = (TRAMPOLINE_ADDRESS / 0x1000) as u8;

// Mailbox field offsets (must match ap_trampoline.asm)
pub const READY_OFFSET: u64 = 0x08;
pub const PAGE_TABLE_OFFSET: u64 = 0x10;
pub const STACK_TOP_OFFSET: u64 = 0x18;
pub const ENTRY_POINT_OFFSET: u64 = 0x20;

/// The trampoline binary, assembled from NASM by build.rs.
static TRAMPOLINE_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ap_trampoline.bin"));

/// Write a u64 value to a field in the trampoline page.
pub fn patch_u64(offset: u64, value: u64) {
    let address = TRAMPOLINE_ADDRESS + offset;
    unsafe { core::ptr::write_volatile(address as *mut u64, value) };
}

/// Read a u64 value from a field in the trampoline page.
pub fn read_u64(offset: u64) -> u64 {
    let address = TRAMPOLINE_ADDRESS + offset;
    unsafe { core::ptr::read_volatile(address as *const u64) }
}

/// Copy the trampoline binary to physical address 0x8000.
/// Must be called once before starting any APs.
pub fn install() {
    let destination = TRAMPOLINE_ADDRESS as *mut u8;

    // zero the page first
    unsafe { core::ptr::write_bytes(destination, 0, 0x1000) };

    // copy the trampoline binary
    unsafe {
        core::ptr::copy_nonoverlapping(
            TRAMPOLINE_BINARY.as_ptr(),
            destination,
            TRAMPOLINE_BINARY.len(),
        );
    }
}
