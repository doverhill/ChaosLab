//! Early boot identity mapping.
//!
//! Scans the UEFI memory map to find all physical address ranges (RAM and MMIO),
//! then creates 2 MiB identity map entries so virtual address == physical address
//! for those ranges. Only allocates L2 page tables for GiB ranges that actually
//! contain memory regions — empty gaps are left unmapped.
//!
//! The identity map lives in L4[0..127] (0 - 64 TiB) and is shared across all
//! processes. Per-process virtual memory starts at L4[128] (64 TiB).
//!
//! ## Page table layout after init
//!
//! - **L4[0..1]**: Identity mapping (virtual = physical). Created here. Some L3/L2
//!   tables may be reused from the bootloader (those physical pages are in
//!   Bootloader-marked memory and must NOT be freed).
//! - **L4[2+]**: Bootloader's physical memory offset mapping + kernel code mapping.
//!   NOT touched by this module. The kernel executes from virtual addresses in
//!   L4[2] (physical_memory_offset + elf_vaddr). The framebuffer is also accessed
//!   through L4[2]'s offset mapping (not identity-mapped, since the framebuffer's
//!   PCI BAR address is not in any UEFI memory region).
//!
//! ## Known gaps
//!
//! - **L4[0]/L3[0]/L2[0]** is skipped (null pointer guard): physical 0-2 MiB is
//!   NOT identity-mapped. Code that accesses physical addresses below 2 MiB via
//!   identity mapping (e.g. AcpiHandler) will page fault.
//! - The framebuffer (physical 0x80000000 on QEMU) is not in any UEFI memory
//!   region, so the 2-3 GiB range is not identity-mapped. Framebuffer access goes
//!   through the bootloader's offset mapping at L4[2].

use bootloader_api::info::MemoryRegions;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::PhysAddr;

use crate::{log, log_println};

const PAGE_SIZE: u64 = 0x1000;
const TWO_MIB: u64 = 0x20_0000;
const ONE_GIB: u64 = 0x4000_0000;
const L4_COVERAGE: u64 = 512 * ONE_GIB; // 512 GiB per L4 entry

/// Maximum L4 index for identity mapping (L4[0..127] = 64 TiB physical half).
const MAX_L4_INDEX: usize = 128;

/// Set up 2 MiB identity mapping for all physical memory and MMIO regions.
pub fn init(physical_memory_offset: u64, memory_regions: &MemoryRegions) {
    // find the highest physical address across all memory regions
    let mut highest_addr: u64 = 0;
    for region in memory_regions.iter() {
        if region.end > highest_addr {
            highest_addr = region.end;
        }
    }

    let l4_count = (((highest_addr + L4_COVERAGE - 1) / L4_COVERAGE) as usize).min(MAX_L4_INDEX);

    log_println!(
        log::SubSystem::Boot,
        log::LogLevel::Information,
        "Identity mapping all physical memory and MMIO (highest address: {:#x}, {} L4 entries)",
        highest_addr,
        l4_count
    );

    let mut bootstrap = BootstrapFrameAllocator::new(memory_regions, physical_memory_offset);

    let (l4_frame, _) = Cr3::read();
    let l4_phys = l4_frame.start_address().as_u64();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L4 page table at physical {:#x}", l4_phys);
    let l4_table = phys_to_table(l4_phys, physical_memory_offset);

    let huge_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE;
    let mut mapped_gibs = 0u64;

    for l4_idx in 0..l4_count {
        let l4_present = l4_table[l4_idx].flags().contains(PageTableFlags::PRESENT);
        let l3_table = get_or_create_table(&mut l4_table[l4_idx], &mut bootstrap, physical_memory_offset);
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L4[{}]: L3 table {} ({})", l4_idx, if l4_present { "reused from bootloader" } else { "newly allocated" }, if l4_present { "bootloader-owned page" } else { "bootstrap frame" });

        for l3_idx in 0usize..512 {
            let gib_start = (l4_idx as u64) * L4_COVERAGE + (l3_idx as u64) * ONE_GIB;
            let gib_end = gib_start + ONE_GIB;

            if !has_region_in_range(memory_regions, gib_start, gib_end) {
                continue;
            }

            let l3_entry = &mut l3_table[l3_idx];

            if l3_entry.flags().contains(PageTableFlags::PRESENT) && l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "  L3[{}]: {:#x}-{:#x} already 1 GiB huge page", l3_idx, gib_start, gib_end);
                mapped_gibs += 1;
                continue;
            }

            let l3_present = l3_entry.flags().contains(PageTableFlags::PRESENT);
            let l2_table = get_or_create_table(l3_entry, &mut bootstrap, physical_memory_offset);
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "  L3[{}]: {:#x}-{:#x} L2 table {} → filling with 2 MiB identity pages", l3_idx, gib_start, gib_end, if l3_present { "reused" } else { "new" });

            for l2_idx in 0usize..512 {
                if l4_idx == 0 && l3_idx == 0 && l2_idx == 0 {
                    continue; // null pointer guard page: physical 0-2 MiB unmapped
                }

                if l2_table[l2_idx].flags().contains(PageTableFlags::PRESENT) && l2_table[l2_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
                    continue;
                }

                let phys = gib_start + (l2_idx as u64) * TWO_MIB;
                l2_table[l2_idx].set_addr(PhysAddr::new(phys), huge_flags);
            }

            mapped_gibs += 1;
        }
    }

    x86_64::instructions::tlb::flush_all();
    log_println!(
        log::SubSystem::Boot,
        log::LogLevel::Information,
        "Identity mapping active ({} GiB ranges mapped with 2 MiB pages, {} bootstrap frames used)",
        mapped_gibs,
        bootstrap.next
    );
}

/// Check whether any memory region overlaps with [start, end).
fn has_region_in_range(memory_regions: &MemoryRegions, start: u64, end: u64) -> bool {
    memory_regions.iter().any(|r| r.start < end && r.end > start)
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
    frames: [u64; 32],
    count: usize,
    next: usize,
    offset: u64,
}

impl BootstrapFrameAllocator {
    fn new(memory_regions: &MemoryRegions, physical_memory_offset: u64) -> Self {
        use bootloader_api::info::MemoryRegionKind;

        let mut frames = [0u64; 32];
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
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Bootstrap allocator: {} frames available below 2 MiB", count);

        BootstrapFrameAllocator { frames, count, next: 0, offset: physical_memory_offset }
    }

    fn alloc_frame(&mut self) -> u64 {
        assert!(self.next < self.count, "bootstrap frame allocator exhausted ({} frames used)", self.next);
        let frame = self.frames[self.next];
        self.next += 1;
        // zero through offset mapping
        unsafe { core::ptr::write_bytes((frame + self.offset) as *mut u8, 0, PAGE_SIZE as usize) };
        frame
    }
}
