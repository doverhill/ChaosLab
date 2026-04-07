//! Early boot identity mapping.
//!
//! Uses the bootloader's Dynamic offset mapping to access page tables, then creates
//! 2 MiB identity map entries so that virtual address == physical address for the
//! first 4 GiB. After this runs, the physical frame allocator can write directly
//! to physical addresses.

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::PhysAddr;

use crate::{log, log_println};

const PAGE_SIZE: u64 = 0x1000;
const TWO_MIB: u64 = 0x20_0000;
const ONE_GIB: u64 = 0x4000_0000;

/// Set up 2 MiB identity mapping for the first 4 GiB of physical address space.
///
/// Allocates page table frames from usable memory below 2 MiB (which the physical
/// allocator will later skip), so there is no double-use conflict.
pub fn init(physical_memory_offset: u64, memory_regions: &MemoryRegions) {
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Setting up identity mapping (0-4 GiB)");

    let mut bootstrap = BootstrapFrameAllocator::new(memory_regions, physical_memory_offset);

    let (l4_frame, _) = Cr3::read();
    let l4_phys = l4_frame.start_address().as_u64();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L4 page table at physical {:#x}", l4_phys);
    let l4_table = phys_to_table(l4_phys, physical_memory_offset);

    // L4[0] covers virtual 0..512 GiB — get or create the L3 table it points to
    let l3_table = get_or_create_table(&mut l4_table[0], &mut bootstrap, physical_memory_offset);

    let huge_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE;

    for l3_idx in 0u64..4 {
        let l3_entry = &mut l3_table[l3_idx as usize];

        // If this is already a 1 GiB huge page, it covers the whole range
        if l3_entry.flags().contains(PageTableFlags::PRESENT) && l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L3[{}]: 1 GiB huge page already present, skipping", l3_idx);
            continue;
        }

        let l2_table = get_or_create_table(l3_entry, &mut bootstrap, physical_memory_offset);

        for l2_idx in 0usize..512 {
            // Keep L2[0] of the first GiB — bootloader's low structures and null pointer guard page
            if l3_idx == 0 && l2_idx == 0 {
                continue;
            }

            // Skip entries that are already 2 MiB huge pages
            if l2_table[l2_idx].flags().contains(PageTableFlags::PRESENT) && l2_table[l2_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
                continue;
            }

            // Replace everything else (absent entries OR bootloader's 4KiB L1 tables
            // that only partially cover the 2MiB range) with a full 2MiB identity map
            let phys = l3_idx * ONE_GIB + (l2_idx as u64) * TWO_MIB;
            l2_table[l2_idx].set_addr(PhysAddr::new(phys), huge_flags);
        }
    }

    x86_64::instructions::tlb::flush_all();
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Identity mapping active (0-4 GiB, 2 MiB pages)");
}

fn phys_to_table(phys: u64, offset: u64) -> &'static mut PageTable {
    unsafe { &mut *((phys + offset) as *mut PageTable) }
}

/// Return the page table pointed to by `entry`, allocating a new zeroed one if absent.
fn get_or_create_table(entry: &mut PageTableEntry, bootstrap: &mut BootstrapFrameAllocator, offset: u64) -> &'static mut PageTable {
    if entry.flags().contains(PageTableFlags::PRESENT) {
        assert!(!entry.flags().contains(PageTableFlags::HUGE_PAGE), "expected table pointer, got huge page");
        return phys_to_table(entry.addr().as_u64(), offset);
    }
    let frame = bootstrap.alloc_frame();
    entry.set_addr(PhysAddr::new(frame), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    phys_to_table(frame, offset)
}

/// Tiny frame allocator for early boot. Grabs usable frames below 2 MiB so they
/// never conflict with the physical allocator (which skips everything < 2 MiB).
struct BootstrapFrameAllocator {
    frames: [u64; 8],
    count: usize,
    next: usize,
    offset: u64,
}

impl BootstrapFrameAllocator {
    fn new(memory_regions: &MemoryRegions, physical_memory_offset: u64) -> Self {
        let mut frames = [0u64; 8];
        let mut count = 0;

        for region in memory_regions.iter() {
            if region.kind != MemoryRegionKind::Usable || count >= frames.len() {
                continue;
            }
            let start = region.start.max(PAGE_SIZE); // skip frame 0
            let end = region.end.min(TWO_MIB);
            if start >= end {
                continue;
            }
            let mut addr = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1); // align up
            while addr + PAGE_SIZE <= end && count < frames.len() {
                frames[count] = addr;
                count += 1;
                addr += PAGE_SIZE;
            }
        }

        assert!(count >= 5, "need at least 5 usable frames below 2 MiB for page tables (found {})", count);
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Bootstrap allocator: {} frames below 2 MiB", count);

        BootstrapFrameAllocator { frames, count, next: 0, offset: physical_memory_offset }
    }

    fn alloc_frame(&mut self) -> u64 {
        assert!(self.next < self.count, "bootstrap frame allocator exhausted");
        let frame = self.frames[self.next];
        self.next += 1;
        // Zero through offset mapping
        unsafe { core::ptr::write_bytes((frame + self.offset) as *mut u8, 0, PAGE_SIZE as usize) };
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Bootstrap frame at {:#x}", frame);
        frame
    }
}
