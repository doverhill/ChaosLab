//! Kernel virtual memory allocation.
//!
//! Manages L4[128..255] — the kernel virtual address space (second quarter
//! of the 48-bit canonical address range: 0x4000_0000_0000 to 0x7FFF_FFFF_FFFF).
//!
//! Provides `allocate_contiguous_pages(count)` which finds unmapped consecutive
//! virtual pages, allocates physical frames for each, maps them, and returns
//! the virtual base address.
//!
//! ## Address space layout
//!
//! - L4[0..127]:   Identity-mapped physical memory (managed by address_space.rs)
//! - L4[128..255]: Kernel virtual memory (managed by this module)
//! - L4[256..511]: Per-process virtual memory (future)

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{PageTable, PageTableFlags};
use x86_64::PhysAddr;

use crate::{log, log_println, physical};

const PAGE_SIZE: u64 = 0x1000;
const ENTRIES_PER_TABLE: usize = 512;
const L4_COVERAGE: u64 = 512 * 1024 * 1024 * 1024; // 512 GiB per L4 entry

/// First L4 index for kernel virtual memory.
const KERNEL_VIRTUAL_L4_START: usize = 128;
/// Last L4 index (exclusive) for kernel virtual memory.
const KERNEL_VIRTUAL_L4_END: usize = 256;

/// Start of the kernel virtual address range.
pub const KERNEL_VIRTUAL_BASE: u64 = (KERNEL_VIRTUAL_L4_START as u64) * L4_COVERAGE;

const TABLE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);
const PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn physical_to_table(physical_address: u64) -> &'static mut PageTable {
    // identity mapping: virtual = physical (for addresses in L4[0..127])
    unsafe { &mut *(physical_address as *mut PageTable) }
}

fn get_l4_table() -> &'static mut PageTable {
    let (l4_frame, _) = Cr3::read();
    physical_to_table(l4_frame.start_address().as_u64())
}

/// Allocate a zeroed page table from the physical allocator.
fn allocate_page_table() -> (&'static mut PageTable, u64) {
    let physical_address = physical::allocate(1).expect("out of memory for page table") as u64;
    let table = physical_to_table(physical_address);
    for entry in table.iter_mut() {
        entry.set_unused();
    }
    (table, physical_address)
}

/// Ensure the L3 table exists for a given L4 index, creating it if needed.
fn ensure_l3_table(l4_table: &mut PageTable, l4_index: usize) -> &'static mut PageTable {
    if !l4_table[l4_index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l4_table[l4_index].set_addr(PhysAddr::new(physical_address), TABLE_FLAGS);
    }
    physical_to_table(l4_table[l4_index].addr().as_u64())
}

/// Ensure the L2 table exists for a given L3 index, creating it if needed.
fn ensure_l2_table(l3_table: &mut PageTable, l3_index: usize) -> &'static mut PageTable {
    assert!(
        !l3_table[l3_index].flags().contains(PageTableFlags::HUGE_PAGE),
        "L3[{}] is a 1 GiB huge page, cannot create L2 subtable", l3_index
    );
    if !l3_table[l3_index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l3_table[l3_index].set_addr(PhysAddr::new(physical_address), TABLE_FLAGS);
    }
    physical_to_table(l3_table[l3_index].addr().as_u64())
}

/// Ensure the L1 table exists for a given L2 index, creating it if needed.
fn ensure_l1_table(l2_table: &mut PageTable, l2_index: usize) -> &'static mut PageTable {
    assert!(
        !l2_table[l2_index].flags().contains(PageTableFlags::HUGE_PAGE),
        "L2[{}] is a 2 MiB huge page, cannot create L1 subtable", l2_index
    );
    if !l2_table[l2_index].flags().contains(PageTableFlags::PRESENT) {
        let (_, physical_address) = allocate_page_table();
        l2_table[l2_index].set_addr(PhysAddr::new(physical_address), TABLE_FLAGS);
    }
    physical_to_table(l2_table[l2_index].addr().as_u64())
}

// ---------------------------------------------------------------------------
// Virtual address decomposition
// ---------------------------------------------------------------------------

fn virtual_to_indices(virtual_address: u64) -> (usize, usize, usize, usize) {
    let l4_index = ((virtual_address >> 39) & 0x1FF) as usize;
    let l3_index = ((virtual_address >> 30) & 0x1FF) as usize;
    let l2_index = ((virtual_address >> 21) & 0x1FF) as usize;
    let l1_index = ((virtual_address >> 12) & 0x1FF) as usize;
    (l4_index, l3_index, l2_index, l1_index)
}

fn indices_to_virtual(l4_index: usize, l3_index: usize, l2_index: usize, l1_index: usize) -> u64 {
    ((l4_index as u64) << 39)
        | ((l3_index as u64) << 30)
        | ((l2_index as u64) << 21)
        | ((l1_index as u64) << 12)
}

// ---------------------------------------------------------------------------
// Check if a 4 KiB page is unmapped
// ---------------------------------------------------------------------------

fn is_page_unmapped(l4_table: &PageTable, virtual_address: u64) -> bool {
    let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);

    let l4_entry = &l4_table[l4_index];
    if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
        return true; // entire L4 subtree is unmapped
    }

    let l3_table = physical_to_table(l4_entry.addr().as_u64());
    let l3_entry = &l3_table[l3_index];
    if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
        return true;
    }
    if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return false; // 1 GiB huge page, mapped
    }

    let l2_table = physical_to_table(l3_entry.addr().as_u64());
    let l2_entry = &l2_table[l2_index];
    if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
        return true;
    }
    if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return false; // 2 MiB huge page, mapped
    }

    let l1_table = physical_to_table(l2_entry.addr().as_u64());
    let l1_entry = &l1_table[l1_index];
    !l1_entry.flags().contains(PageTableFlags::PRESENT)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Find `page_count` contiguous unmapped 4 KiB virtual pages in the kernel
/// virtual range (L4[128..255]), allocate a physical frame for each, map them,
/// and return the virtual base address.
///
/// Returns None if no contiguous range is found or if physical allocation fails.
///
/// NOTE: This is a naive brute-force linear scan. It should be replaced with
/// a proper free-range tracker (e.g. a bitmap or tree) once the kernel matures.
pub fn allocate_contiguous_pages(page_count: usize) -> Option<*mut u8> {
    assert!(page_count > 0, "cannot allocate 0 pages");

    let l4_table = get_l4_table();

    // brute-force scan: try each possible starting address in L4[128..256]
    // TODO: replace with a proper free-range data structure
    let start_virtual = KERNEL_VIRTUAL_BASE;
    let end_virtual = (KERNEL_VIRTUAL_L4_END as u64) * L4_COVERAGE;
    let mut candidate = start_virtual;

    while candidate + (page_count as u64 * PAGE_SIZE) <= end_virtual {
        // check if all pages in [candidate, candidate + page_count * PAGE_SIZE) are unmapped
        let mut all_free = true;
        for page_offset in 0..page_count as u64 {
            let virtual_address = candidate + page_offset * PAGE_SIZE;
            if !is_page_unmapped(l4_table, virtual_address) {
                // skip past this mapped page
                candidate = virtual_address + PAGE_SIZE;
                all_free = false;
                break;
            }
        }

        if !all_free {
            continue;
        }

        // found a contiguous unmapped range — allocate physical frames and map them
        for page_offset in 0..page_count as u64 {
            let virtual_address = candidate + page_offset * PAGE_SIZE;
            let physical_frame = physical::allocate(1);
            if physical_frame.is_none() {
                // out of physical memory — unmap what we already mapped
                // TODO: proper rollback (free physical frames and unmap pages)
                log_println!(log::SubSystem::KernelMemory, log::LogLevel::Error,
                    "Out of physical memory while mapping virtual pages at {:#x}", virtual_address);
                return None;
            }
            let physical_address = physical_frame.unwrap() as u64;

            // map this virtual page to the physical frame
            let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);
            let l3_table = ensure_l3_table(l4_table, l4_index);
            let l2_table = ensure_l2_table(l3_table, l3_index);
            let l1_table = ensure_l1_table(l2_table, l2_index);
            l1_table[l1_index].set_addr(PhysAddr::new(physical_address), PAGE_FLAGS);
        }

        log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug,
            "Mapped {} virtual pages at {:#x}", page_count, candidate);
        return Some(candidate as *mut u8);
    }

    log_println!(log::SubSystem::KernelMemory, log::LogLevel::Error,
        "Could not find {} contiguous unmapped virtual pages", page_count);
    None
}

/// Free a contiguous range of virtual pages previously allocated with
/// `allocate_contiguous_pages`. Unmaps the pages and returns the physical
/// frames to the physical allocator.
pub fn free_contiguous_pages(virtual_base: *mut u8, page_count: usize) {
    let l4_table = get_l4_table();
    let base = virtual_base as u64;

    for page_offset in 0..page_count as u64 {
        let virtual_address = base + page_offset * PAGE_SIZE;
        let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);

        let l4_entry = &l4_table[l4_index];
        if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
            continue;
        }
        let l3_table = physical_to_table(l4_entry.addr().as_u64());
        let l3_entry = &l3_table[l3_index];
        if !l3_entry.flags().contains(PageTableFlags::PRESENT) || l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            continue;
        }
        let l2_table = physical_to_table(l3_entry.addr().as_u64());
        let l2_entry = &l2_table[l2_index];
        if !l2_entry.flags().contains(PageTableFlags::PRESENT) || l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            continue;
        }
        let l1_table = physical_to_table(l2_entry.addr().as_u64());
        let l1_entry = &mut l1_table[l1_index];

        if l1_entry.flags().contains(PageTableFlags::PRESENT) {
            let physical_address = l1_entry.addr().as_u64();
            l1_entry.set_unused();
            physical::free(physical_address as *mut u8, 1);
        }
    }

    // flush TLB for the freed range
    for page_offset in 0..page_count as u64 {
        let virtual_address = base + page_offset * PAGE_SIZE;
        x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(virtual_address));
    }

    log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug,
        "Freed {} virtual pages at {:#x}", page_count, base);
}
