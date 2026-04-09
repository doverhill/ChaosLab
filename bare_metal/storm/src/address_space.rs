//! Per-process address space management.
//!
//! Each `AddressSpace` owns an L4 page table. The kernel half (L4[0..255])
//! is shared across all address spaces by copying entries from the kernel L4.
//! The user half (L4[256..511]) is private to each process.

use x86_64::structures::paging::page_table::PageTableFlags;
use x86_64::PhysAddr;

use crate::page_tables::{
    allocate_page_table, ensure_l1_table, ensure_l2_table, ensure_l3_table, get_current_l4_table, is_page_unmapped, physical_to_table, virtual_to_indices, L4_COVERAGE, PAGE_SIZE, USER_PAGE_FLAGS,
    USER_TABLE_FLAGS, USER_VIRTUAL_L4_END, USER_VIRTUAL_L4_START,
};
use crate::{log, log_println, physical_memory};

/// Start of the user virtual address range.
const USER_VIRTUAL_BASE: u64 = (USER_VIRTUAL_L4_START as u64) * L4_COVERAGE;

/// A per-process address space backed by its own L4 page table.
///
/// The kernel half (L4[0..255]) is copied from the current kernel L4 at
/// creation time. Because all kernel L3 tables are pre-allocated and shared,
/// kernel mappings created after construction are automatically visible.
///
/// The user half (L4[256..511]) starts empty and is managed by
/// `allocate_user_pages` / `free_user_pages`.
pub struct AddressSpace {
    l4_physical_address: u64,
}

impl AddressSpace {
    /// Create a new address space. Allocates an L4 table, copies L4[0..255]
    /// from the current kernel L4 (sharing the kernel page tables), and
    /// leaves L4[256..511] empty for user-space mappings.
    pub fn new() -> Self {
        let kernel_l4 = get_current_l4_table();
        let (new_l4, new_l4_physical) = allocate_page_table();

        // Copy the kernel half (identity mapping + kernel virtual memory).
        // These entries point to shared L3 tables, so any future kernel
        // mappings (e.g. new kernel virtual allocations) are automatically
        // visible in this address space.
        for index in 0..256 {
            if kernel_l4[index].flags().contains(PageTableFlags::PRESENT) {
                new_l4[index].set_addr(kernel_l4[index].addr(), kernel_l4[index].flags());
            }
        }

        // L4[256..511] stays zeroed (set_unused by allocate_page_table)

        log_println!(
            log::SubSystem::KernelMemory,
            log::LogLevel::Debug,
            "Created address space with L4 at physical {:#x}",
            new_l4_physical
        );

        AddressSpace { l4_physical_address: new_l4_physical }
    }

    /// Return the physical address of this address space's L4 page table.
    /// Used when switching CR3 to activate this address space.
    pub fn l4_physical_address(&self) -> u64 {
        self.l4_physical_address
    }

    /// Allocate `page_count` contiguous 4 KiB virtual pages in the user half
    /// (L4[256..511]). Allocates physical frames for each page and maps them
    /// with user-accessible flags.
    ///
    /// Returns the virtual base address, or None if no contiguous range is
    /// found or physical allocation fails.
    ///
    /// NOTE: Brute-force linear scan; should be replaced with a proper
    /// free-range tracker once the kernel matures.
    pub fn allocate_user_pages(&self, page_count: usize) -> Option<u64> {
        assert!(page_count > 0, "cannot allocate 0 pages");

        let l4_table = physical_to_table(self.l4_physical_address);
        let end_virtual = (USER_VIRTUAL_L4_END as u64) * L4_COVERAGE;
        let mut candidate = USER_VIRTUAL_BASE;

        while candidate + (page_count as u64 * PAGE_SIZE) <= end_virtual {
            let mut all_free = true;
            for page_offset in 0..page_count as u64 {
                let virtual_address = candidate + page_offset * PAGE_SIZE;
                if !is_page_unmapped(l4_table, virtual_address) {
                    candidate = virtual_address + PAGE_SIZE;
                    all_free = false;
                    break;
                }
            }

            if !all_free {
                continue;
            }

            // found a contiguous unmapped range — allocate and map
            for page_offset in 0..page_count as u64 {
                let virtual_address = candidate + page_offset * PAGE_SIZE;
                let physical_frame = physical_memory::allocate(1);
                if physical_frame.is_none() {
                    log_println!(
                        log::SubSystem::KernelMemory,
                        log::LogLevel::Error,
                        "Out of physical memory while mapping user pages at {:#x}",
                        virtual_address
                    );
                    // TODO: proper rollback (free already-mapped pages)
                    return None;
                }
                let physical_address = physical_frame.unwrap() as u64;

                let (l4_index, l3_index, l2_index, l1_index) = virtual_to_indices(virtual_address);
                let l3_table = ensure_l3_table(l4_table, l4_index, USER_TABLE_FLAGS);
                let l2_table = ensure_l2_table(l3_table, l3_index, USER_TABLE_FLAGS);
                let l1_table = ensure_l1_table(l2_table, l2_index, USER_TABLE_FLAGS);
                l1_table[l1_index].set_addr(PhysAddr::new(physical_address), USER_PAGE_FLAGS);
            }

            log_println!(
                log::SubSystem::KernelMemory,
                log::LogLevel::Debug,
                "Mapped {} user pages at {:#x} in address space {:#x}",
                page_count,
                candidate,
                self.l4_physical_address
            );
            return Some(candidate);
        }

        log_println!(
            log::SubSystem::KernelMemory,
            log::LogLevel::Error,
            "Could not find {} contiguous unmapped user pages",
            page_count
        );
        None
    }

    /// Free a contiguous range of user virtual pages previously allocated with
    /// `allocate_user_pages`. Unmaps the pages and returns the physical frames
    /// to the physical allocator.
    pub fn free_user_pages(&self, virtual_base: u64, page_count: usize) {
        let l4_table = physical_to_table(self.l4_physical_address);
        let base = virtual_base;

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

        log_println!(
            log::SubSystem::KernelMemory,
            log::LogLevel::Debug,
            "Freed {} user pages at {:#x} in address space {:#x}",
            page_count,
            base,
            self.l4_physical_address
        );
    }
}
