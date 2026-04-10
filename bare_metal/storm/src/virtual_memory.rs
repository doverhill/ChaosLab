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
//! - L4[0..127]:   Identity-mapped physical memory (managed by memory_setup.rs)
//! - L4[128..255]: Kernel virtual memory (managed by this module)
//! - L4[256..511]: Per-process virtual memory (managed by address_space.rs)

use x86_64::structures::paging::page_table::PageTableFlags;
use x86_64::PhysAddr;

use crate::arch::page_tables::{
    ensure_l1_table, ensure_l2_table, ensure_l3_table, get_current_l4_table, is_page_unmapped, physical_to_table, virtual_to_indices, KERNEL_VIRTUAL_BASE, KERNEL_VIRTUAL_L4_END, L4_COVERAGE,
    PAGE_FLAGS, PAGE_SIZE, TABLE_FLAGS,
};
use crate::{log, log_println, physical_memory};

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

    let l4_table = get_current_l4_table();

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
            let physical_frame = physical_memory::allocate(1);
            if physical_frame.is_none() {
                // out of physical memory — unmap what we already mapped
                // TODO: proper rollback (free physical frames and unmap pages)
                log_println!(log::SubSystem::KernelMemory, log::LogLevel::Error, "Out of physical memory while mapping virtual pages at {:#x}", virtual_address);
                return None;
            }
            let physical_address = physical_frame.unwrap() as u64;

            // map this virtual page to the physical frame
            let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);
            let l3_table = ensure_l3_table(l4_table, l4_index, TABLE_FLAGS);
            let l2_table = ensure_l2_table(l3_table, l3_index, TABLE_FLAGS);
            let l1_table = ensure_l1_table(l2_table, l2_index, TABLE_FLAGS);
            l1_table[l1_index].set_addr(PhysAddr::new(physical_address), PAGE_FLAGS);
        }

        log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Mapped {} virtual pages at {:#x}", page_count, candidate);
        return Some(candidate as *mut u8);
    }

    log_println!(log::SubSystem::KernelMemory, log::LogLevel::Error, "Could not find {} contiguous unmapped virtual pages", page_count);
    None
}

/// Free a contiguous range of virtual pages previously allocated with
/// `allocate_contiguous_pages`. Unmaps the pages and returns the physical
/// frames to the physical allocator.
pub fn free_contiguous_pages(virtual_base: *mut u8, page_count: usize) {
    let l4_table = get_current_l4_table();
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
            physical_memory::free(physical_address as *mut u8, 1);
        }
    }

    // flush TLB for the freed range
    for page_offset in 0..page_count as u64 {
        let virtual_address = base + page_offset * PAGE_SIZE;
        x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(virtual_address));
    }

    log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Freed {} virtual pages at {:#x}", page_count, base);
}
