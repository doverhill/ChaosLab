//! Low-level page table utilities (pure mechanics, no policy).
//!
//! Provides constants, address decomposition, page table traversal, and
//! allocation helpers used by `memory_setup`, `virtual_memory`, and
//! `address_space`.

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{PageTable, PageTableFlags};
use x86_64::PhysAddr;

use crate::physical_memory;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const PAGE_SIZE: u64 = 0x1000;
pub const TWO_MIB: u64 = 0x20_0000;
pub const ONE_GIB: u64 = 0x4000_0000;
pub const L4_COVERAGE: u64 = 512 * ONE_GIB;

pub const TABLE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);
pub const HUGE_FLAGS: PageTableFlags = TABLE_FLAGS.union(PageTableFlags::HUGE_PAGE);
pub const PAGE_FLAGS: PageTableFlags = TABLE_FLAGS;
pub const USER_TABLE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE).union(PageTableFlags::USER_ACCESSIBLE);
pub const USER_PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE).union(PageTableFlags::USER_ACCESSIBLE);

// ---------------------------------------------------------------------------
// Address space layout
// ---------------------------------------------------------------------------

/// Last L4 index (exclusive) for identity-mapped physical memory.
pub const MAX_IDENTITY_L4_INDEX: usize = 128;

/// First L4 index for kernel virtual memory.
pub const KERNEL_VIRTUAL_L4_START: usize = 128;
/// Last L4 index (exclusive) for kernel virtual memory.
pub const KERNEL_VIRTUAL_L4_END: usize = 256;

/// First L4 index for per-process user virtual memory.
pub const USER_VIRTUAL_L4_START: usize = 256;
/// Last L4 index (exclusive) for per-process user virtual memory.
pub const USER_VIRTUAL_L4_END: usize = 512;

/// Make a canonical 64-bit virtual address from a 48-bit value.
/// Sign-extends bit 47 through bits 63:48 as required by x86_64.
pub const fn canonicalize(address_48bit: u64) -> u64 {
    if address_48bit & (1u64 << 47) != 0 {
        address_48bit | 0xFFFF_0000_0000_0000
    } else {
        address_48bit
    }
}

/// A contiguous range of canonical virtual addresses defined by L4 indices.
/// Handles sign extension and provides range-check helpers so callers
/// don't need to reason about canonical arithmetic.
pub struct VirtualAddressRange {
    pub l4_start: usize,
    pub l4_end: usize,
    pub base: u64,
}

impl VirtualAddressRange {
    pub const fn new(l4_start: usize, l4_end: usize) -> Self {
        Self {
            l4_start,
            l4_end,
            base: canonicalize((l4_start as u64) * L4_COVERAGE),
        }
    }

    /// True if `address` falls within this range.
    /// Works correctly across the canonical hole by comparing L4 indices.
    pub fn contains(&self, address: u64) -> bool {
        let l4_index = ((address >> 39) & 0x1FF) as usize;
        l4_index >= self.l4_start && l4_index < self.l4_end
    }

    /// True if the entire region [address, address+size) fits in this range.
    pub fn contains_region(&self, address: u64, size: u64) -> bool {
        if size == 0 { return self.contains(address); }
        let last = address.wrapping_add(size - 1);
        self.contains(address) && self.contains(last)
    }

    /// Number of L4 entries spanned.
    pub const fn l4_count(&self) -> usize {
        self.l4_end - self.l4_start
    }
}

pub const KERNEL_VIRTUAL_RANGE: VirtualAddressRange = VirtualAddressRange::new(KERNEL_VIRTUAL_L4_START, KERNEL_VIRTUAL_L4_END);
pub const USER_VIRTUAL_RANGE: VirtualAddressRange = VirtualAddressRange::new(USER_VIRTUAL_L4_START, USER_VIRTUAL_L4_END);

/// Start of the kernel virtual address range (canonical).
pub const KERNEL_VIRTUAL_BASE: u64 = KERNEL_VIRTUAL_RANGE.base;
/// Start of the user virtual address range (canonical).
pub const USER_VIRTUAL_BASE: u64 = USER_VIRTUAL_RANGE.base;

// ---------------------------------------------------------------------------
// Physical-to-table conversions
// ---------------------------------------------------------------------------

/// Convert a physical address to a mutable page table reference using identity
/// mapping (virtual = physical). Only valid after identity mapping is active.
pub fn physical_to_table(physical_address: u64) -> &'static mut PageTable {
    unsafe { &mut *(physical_address as *mut PageTable) }
}

/// Convert a physical address to a mutable page table reference using an
/// explicit offset. Required during early boot before identity mapping is
/// complete (the bootloader provides a physical_memory_offset).
pub fn physical_to_table_with_offset(physical_address: u64, offset: u64) -> &'static mut PageTable {
    unsafe { &mut *((physical_address + offset) as *mut PageTable) }
}

// ---------------------------------------------------------------------------
// Current L4 table
// ---------------------------------------------------------------------------

/// Read CR3 and return a mutable reference to the current L4 page table.
/// Uses identity mapping.
pub fn get_current_l4_table() -> &'static mut PageTable {
    let (l4_frame, _) = Cr3::read();
    physical_to_table(l4_frame.start_address().as_u64())
}

// ---------------------------------------------------------------------------
// Page table allocation
// ---------------------------------------------------------------------------

/// Allocate a zeroed page table from the physical allocator.
/// Returns (table reference via identity mapping, physical address).
pub fn allocate_page_table() -> (&'static mut PageTable, u64) {
    let physical_address = physical_memory::allocate(1).expect("out of memory for page table") as u64;
    let table = physical_to_table(physical_address);
    for entry in table.iter_mut() {
        entry.set_unused();
    }
    (table, physical_address)
}

// ---------------------------------------------------------------------------
// Ensure subtable helpers
// ---------------------------------------------------------------------------

/// Ensure the L3 table exists for a given L4 index, creating it if needed.
pub fn ensure_l3_table(l4_table: &mut PageTable, index: usize, flags: PageTableFlags) -> &'static mut PageTable {
    if !l4_table[index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l4_table[index].set_addr(PhysAddr::new(physical_address), flags);
    }
    physical_to_table(l4_table[index].addr().as_u64())
}

/// Ensure the L2 table exists for a given L3 index, creating it if needed.
pub fn ensure_l2_table(l3_table: &mut PageTable, index: usize, flags: PageTableFlags) -> &'static mut PageTable {
    assert!(
        !l3_table[index].flags().contains(PageTableFlags::HUGE_PAGE),
        "L3[{}] is a 1 GiB huge page, cannot create L2 subtable", index
    );
    if !l3_table[index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l3_table[index].set_addr(PhysAddr::new(physical_address), flags);
    }
    physical_to_table(l3_table[index].addr().as_u64())
}

/// Ensure the L1 table exists for a given L2 index, creating it if needed.
pub fn ensure_l1_table(l2_table: &mut PageTable, index: usize, flags: PageTableFlags) -> &'static mut PageTable {
    assert!(
        !l2_table[index].flags().contains(PageTableFlags::HUGE_PAGE),
        "L2[{}] is a 2 MiB huge page, cannot create L1 subtable", index
    );
    if !l2_table[index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l2_table[index].set_addr(PhysAddr::new(physical_address), flags);
    }
    physical_to_table(l2_table[index].addr().as_u64())
}

// ---------------------------------------------------------------------------
// Address decomposition
// ---------------------------------------------------------------------------

/// Decompose a virtual address into (l4_index, l3_index, l2_index, l1_index).
pub fn virtual_to_indices(virtual_address: u64) -> (usize, usize, usize, usize) {
    let l4_index = ((virtual_address >> 39) & 0x1FF) as usize;
    let l3_index = ((virtual_address >> 30) & 0x1FF) as usize;
    let l2_index = ((virtual_address >> 21) & 0x1FF) as usize;
    let l1_index = ((virtual_address >> 12) & 0x1FF) as usize;
    (l4_index, l3_index, l2_index, l1_index)
}

/// Check whether a 4 KiB page is unmapped in the given L4 table.
pub fn is_page_unmapped(l4_table: &PageTable, virtual_address: u64) -> bool {
    let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);

    let l4_entry = &l4_table[l4_index];
    if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
        return true;
    }

    let l3_table = physical_to_table(l4_entry.addr().as_u64());
    let l3_entry = &l3_table[l3_index];
    if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
        return true;
    }
    if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return false;
    }

    let l2_table = physical_to_table(l3_entry.addr().as_u64());
    let l2_entry = &l2_table[l2_index];
    if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
        return true;
    }
    if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return false;
    }

    let l1_table = physical_to_table(l2_entry.addr().as_u64());
    let l1_entry = &l1_table[l1_index];
    !l1_entry.flags().contains(PageTableFlags::PRESENT)
}

// ---------------------------------------------------------------------------
// Page table walk
// ---------------------------------------------------------------------------

/// Walk the page tables to translate a virtual address to its physical address.
/// Uses an explicit physical_memory_offset for the walk (supports both early
/// boot and post-identity-mapping contexts).
/// Returns None if the address is not mapped.
pub fn virtual_to_physical(virtual_address: u64, physical_memory_offset: u64) -> Option<u64> {
    let (l4_frame, _) = Cr3::read();
    let l4 = physical_to_table_with_offset(l4_frame.start_address().as_u64(), physical_memory_offset);

    let l4_index = ((virtual_address >> 39) & 0x1FF) as usize;
    let l3_index = ((virtual_address >> 30) & 0x1FF) as usize;
    let l2_index = ((virtual_address >> 21) & 0x1FF) as usize;
    let l1_index = ((virtual_address >> 12) & 0x1FF) as usize;
    let page_offset = virtual_address & 0xFFF;

    let l4_entry = &l4[l4_index];
    if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    let l3 = physical_to_table_with_offset(l4_entry.addr().as_u64(), physical_memory_offset);
    let l3_entry = &l3[l3_index];
    if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }
    if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return Some(l3_entry.addr().as_u64() + (virtual_address & (ONE_GIB - 1)));
    }

    let l2 = physical_to_table_with_offset(l3_entry.addr().as_u64(), physical_memory_offset);
    let l2_entry = &l2[l2_index];
    if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }
    if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return Some(l2_entry.addr().as_u64() + (virtual_address & (TWO_MIB - 1)));
    }

    let l1 = physical_to_table_with_offset(l2_entry.addr().as_u64(), physical_memory_offset);
    let l1_entry = &l1[l1_index];
    if !l1_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }
    Some(l1_entry.addr().as_u64() + page_offset)
}
