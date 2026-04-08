//! Identity mapping and bootloader decoupling.
//!
//! ## Boot-time identity mapping (`init`)
//!
//! Scans the UEFI memory map and creates 2 MiB identity map entries
//! (virtual = physical) for every GiB range containing a memory region.
//! The identity map lives in L4[0..127]; per-process virtual space starts
//! at L4[128].
//!
//! ## Post-allocator decoupling (`decouple_from_bootloader`)
//!
//! Once the physical allocator is ready, this module:
//! 1. Fixes the null guard from 2 MiB down to 4 KiB (page 0 only)
//! 2. Identity-maps the framebuffer (not in any UEFI memory region)
//! 3. Walks L4[2+] (bootloader page tables), creates new tables from our
//!    allocator preserving kernel 4 KiB leaf pages, drops offset mapping
//! 4. Replaces the 2 reused bootloader L3/L2 at L4[0..1]
//! 5. Returns the set of kernel leaf pages (still in use, must not free)
//!
//! After decoupling, NO page table pages are in bootloader memory.

use alloc::vec::Vec;
use bootloader_api::info::MemoryRegions;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::PhysAddr;

use crate::{log, log_println, physical};

const PAGE_SIZE: u64 = 0x1000;
const TWO_MIB: u64 = 0x20_0000;
const ONE_GIB: u64 = 0x4000_0000;
const L4_COVERAGE: u64 = 512 * ONE_GIB;
const MAX_L4_INDEX: usize = 128;

const TABLE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);
const HUGE_FLAGS: PageTableFlags = TABLE_FLAGS.union(PageTableFlags::HUGE_PAGE);
const PAGE_FLAGS: PageTableFlags = TABLE_FLAGS;

// ---------------------------------------------------------------------------
// Boot-time identity mapping (called before allocator is ready)
// ---------------------------------------------------------------------------

/// Set up 2 MiB identity mapping for all physical memory and MMIO regions.
/// Called early in boot before the physical allocator exists.
pub fn init(physical_memory_offset: u64, memory_regions: &MemoryRegions) {
    let mut highest_addr: u64 = 0;
    for region in memory_regions.iter() {
        if region.end > highest_addr {
            highest_addr = region.end;
        }
    }

    let l4_count = (((highest_addr + L4_COVERAGE - 1) / L4_COVERAGE) as usize).min(MAX_L4_INDEX);

    log_println!(
        log::SubSystem::Boot, log::LogLevel::Information,
        "Identity mapping all physical memory and MMIO (highest address: {:#x}, {} L4 entries)",
        highest_addr, l4_count
    );

    let mut bootstrap = BootstrapFrameAllocator::new(memory_regions, physical_memory_offset);

    let (l4_frame, _) = Cr3::read();
    let l4_phys = l4_frame.start_address().as_u64();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L4 page table at physical {:#x}", l4_phys);
    let l4_table = phys_to_table(l4_phys, physical_memory_offset);

    let mut mapped_gibs = 0u64;

    for l4_idx in 0..l4_count {
        let l4_present = l4_table[l4_idx].flags().contains(PageTableFlags::PRESENT);
        let l3_table = get_or_create_table(&mut l4_table[l4_idx], &mut bootstrap, physical_memory_offset);
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "L4[{}]: L3 table {} ({})",
            l4_idx,
            if l4_present { "reused from bootloader" } else { "newly allocated" },
            if l4_present { "bootloader-owned page" } else { "bootstrap frame" }
        );

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
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "  L3[{}]: {:#x}-{:#x} L2 table {} → filling with 2 MiB identity pages",
                l3_idx, gib_start, gib_end, if l3_present { "reused" } else { "new" }
            );

            for l2_idx in 0usize..512 {
                // null pointer guard: skip the ENTIRE first 2 MiB for now.
                // decouple_from_bootloader() will refine this to 4 KiB later.
                if l4_idx == 0 && l3_idx == 0 && l2_idx == 0 {
                    continue;
                }

                if l2_table[l2_idx].flags().contains(PageTableFlags::PRESENT) && l2_table[l2_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
                    continue;
                }

                let phys = gib_start + (l2_idx as u64) * TWO_MIB;
                l2_table[l2_idx].set_addr(PhysAddr::new(phys), HUGE_FLAGS);
            }

            mapped_gibs += 1;
        }
    }

    x86_64::instructions::tlb::flush_all();
    log_println!(
        log::SubSystem::Boot, log::LogLevel::Information,
        "Identity mapping active ({} GiB ranges mapped with 2 MiB pages, {} bootstrap frames used)",
        mapped_gibs, bootstrap.next
    );
}

// ---------------------------------------------------------------------------
// Post-allocator: decouple from bootloader page tables
// ---------------------------------------------------------------------------

/// Allocate a zeroed page table from the physical allocator.
/// Returns (virtual pointer, physical address).
fn alloc_table() -> (&'static mut PageTable, u64) {
    let phys = physical::allocate(1).expect("out of memory for page table") as u64;
    // identity mapping: virtual = physical for pages above 2 MiB
    let table = unsafe { &mut *(phys as *mut PageTable) };
    for entry in table.iter_mut() {
        entry.set_unused();
    }
    (table, phys)
}

/// Replace all bootloader-owned page tables with our own allocations.
/// After this call, no page table page is in Bootloader-marked memory.
///
/// Returns a Vec of physical addresses of kernel leaf pages (code, data,
/// stack) that are still in use and must NOT be freed.
pub fn decouple_from_bootloader(physical_memory_offset: u64, framebuffer_physical: u64, framebuffer_size: usize) -> Vec<u64> {
    let (l4_frame, _) = Cr3::read();
    let l4_phys = l4_frame.start_address().as_u64();
    let l4_table = phys_to_table(l4_phys, physical_memory_offset);

    // Only track kernel leaf pages (code, data, stack) — these must NOT be
    // freed during bootloader memory reclamation. Old page table pages don't
    // need tracking: after TLB flush, main.rs walks the bootloader regions
    // and frees everything not in this set.
    let mut kernel_leaf_pages: Vec<u64> = Vec::with_capacity(256);

    // ---- Phase A: fix null guard to 4 KiB only ----
    fix_null_guard(l4_table, physical_memory_offset);

    // ---- Phase B1: identity-map the framebuffer and switch pointer ----
    // MUST happen before clearing L4[4+] which contains the old offset mapping.
    if framebuffer_size > 0 {
        map_physical_range(l4_table, framebuffer_physical, framebuffer_size as u64, physical_memory_offset);
        // switch framebuffer pointer from offset mapping to identity mapping
        // BEFORE we clear the offset mapping entries
        crate::log::remap_framebuffer(framebuffer_physical);
        log_println!(log::SubSystem::Boot, log::LogLevel::Information,
            "Framebuffer identity-mapped at {:#x} ({} KiB)", framebuffer_physical, framebuffer_size / 1024);
    }

    // ---- Phase B2: walk L4[2..512], replace bootloader page tables ----
    // The kernel code is at L4[2] (virtual_address_offset = 0x10000000000),
    // the kernel stack may be at L4[3]. Everything else (L4[4+]) is the
    // bootloader's offset mapping which is redundant with our identity mapping.
    // Rebuild L4[2..4] preserving 4 KiB leaf pages; clear L4[4..512].
    for l4_index in 2..512usize {
        if !l4_table[l4_index].flags().contains(PageTableFlags::PRESENT) {
            continue;
        }

        if l4_index < 4 {
            let old_l3_phys = l4_table[l4_index].addr().as_u64();
            let old_l3 = phys_to_table(old_l3_phys, physical_memory_offset);
            if subtree_has_4k_leaves(old_l3, physical_memory_offset) {
                let (new_l3, new_l3_phys) = alloc_table();
                let kept = rebuild_l3_keeping_4k(old_l3, new_l3, physical_memory_offset, &mut kernel_leaf_pages);
                l4_table[l4_index].set_addr(PhysAddr::new(new_l3_phys), TABLE_FLAGS);
                log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
                    "L4[{}]: rebuilt with our page tables ({} kernel leaf pages kept)", l4_index, kept);
                continue;
            }
        }

        // offset mapping or empty bootloader entry — clear it
        l4_table[l4_index].set_unused();
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "L4[{}]: bootloader mapping cleared", l4_index);
    }

    // ---- Phase B3: replace reused bootloader L3/L2 at L4[0..1] ----
    replace_reused_bootloader_tables(l4_table, physical_memory_offset);

    x86_64::instructions::tlb::flush_all();

    log_println!(log::SubSystem::Boot, log::LogLevel::Information,
        "Decoupled from bootloader: {} kernel leaf pages preserved (old page table pages will be reclaimed)",
        kernel_leaf_pages.len());

    kernel_leaf_pages
}

// ---------------------------------------------------------------------------
// Phase A: fix null guard
// ---------------------------------------------------------------------------

/// Replace the 2 MiB skip at L2[0] with an L1 table that maps 0x1000-0x1FFFFF
/// (pages 1-511) and leaves page 0 unmapped (null pointer guard).
fn fix_null_guard(l4_table: &mut PageTable, offset: u64) {
    let l3_table = {
        let entry = &l4_table[0];
        assert!(entry.flags().contains(PageTableFlags::PRESENT));
        phys_to_table(entry.addr().as_u64(), offset)
    };
    let l2_table = {
        let entry = &l3_table[0];
        assert!(entry.flags().contains(PageTableFlags::PRESENT));
        phys_to_table(entry.addr().as_u64(), offset)
    };

    // L2[0] should currently be empty (we skipped it in init)
    let (l1_table, l1_phys) = alloc_table();

    // map pages 1-511 (physical 0x1000-0x1FFFFF) with identity mapping
    for i in 1..512usize {
        let phys = (i as u64) * PAGE_SIZE;
        l1_table[i].set_addr(PhysAddr::new(phys), PAGE_FLAGS);
    }
    // page 0 stays absent → null pointer dereference causes page fault

    l2_table[0].set_addr(PhysAddr::new(l1_phys), TABLE_FLAGS);
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
        "Null guard: L2[0] now has L1 table, page 0 unmapped, 0x1000-0x1FFFFF identity-mapped");
}

// ---------------------------------------------------------------------------
// Phase B1: identity-map arbitrary physical range
// ---------------------------------------------------------------------------

/// Ensure a physical address range is identity-mapped using 2 MiB huge pages.
/// Creates any missing L3/L2 tables along the way.
fn map_physical_range(l4_table: &mut PageTable, phys_start: u64, size: u64, offset: u64) {
    let start_2m = phys_start & !(TWO_MIB - 1);
    let end = phys_start + size;
    let mut addr = start_2m;
    while addr < end {
        let l4_idx = ((addr / L4_COVERAGE) as usize).min(511);
        let l3_idx = ((addr % L4_COVERAGE) / ONE_GIB) as usize;
        let l2_idx = ((addr % ONE_GIB) / TWO_MIB) as usize;

        // ensure L3 table exists
        if !l4_table[l4_idx].flags().contains(PageTableFlags::PRESENT) {
            let (_, new_phys) = alloc_table();
            l4_table[l4_idx].set_addr(PhysAddr::new(new_phys), TABLE_FLAGS);
        }
        let l3_table = phys_to_table(l4_table[l4_idx].addr().as_u64(), offset);

        // ensure L2 table exists (skip if already a 1 GiB huge page)
        if l3_table[l3_idx].flags().contains(PageTableFlags::PRESENT) && l3_table[l3_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
            addr += ONE_GIB; // already covered
            continue;
        }
        if !l3_table[l3_idx].flags().contains(PageTableFlags::PRESENT) {
            let (_, new_phys) = alloc_table();
            l3_table[l3_idx].set_addr(PhysAddr::new(new_phys), TABLE_FLAGS);
        }
        let l2_table = phys_to_table(l3_table[l3_idx].addr().as_u64(), offset);

        // set 2 MiB identity page if not already present
        if !l2_table[l2_idx].flags().contains(PageTableFlags::PRESENT) {
            l2_table[l2_idx].set_addr(PhysAddr::new(addr), HUGE_FLAGS);
        }

        addr += TWO_MIB;
    }
}

// ---------------------------------------------------------------------------
// Phase B2: walk and rebuild L4[2+] page tables
// ---------------------------------------------------------------------------

/// Check if any L1 (4 KiB) leaf pages exist in this L3 subtree.
fn subtree_has_4k_leaves(l3: &PageTable, offset: u64) -> bool {
    for l3_entry in l3.iter() {
        if !l3_entry.flags().contains(PageTableFlags::PRESENT) || l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            continue;
        }
        let l2 = phys_to_table(l3_entry.addr().as_u64(), offset);
        for l2_entry in l2.iter() {
            if !l2_entry.flags().contains(PageTableFlags::PRESENT) || l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                continue;
            }
            // L2 entry points to L1 table → has 4 KiB pages
            return true;
        }
    }
    false
}

/// Rebuild an L3 subtree, keeping only 4 KiB leaf mappings.
/// Allocates new L2/L1 tables from our allocator.
/// Returns the number of leaf pages kept.
fn rebuild_l3_keeping_4k(
    old_l3: &PageTable, new_l3: &mut PageTable, offset: u64,
    kernel_pages: &mut Vec<u64>,
) -> usize {
    let mut kept = 0;
    for l3_idx in 0..512usize {
        let old_l3_entry = &old_l3[l3_idx];
        if !old_l3_entry.flags().contains(PageTableFlags::PRESENT) {
            continue;
        }
        if old_l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            // 1 GiB huge page = offset mapping, drop it
            continue;
        }

        let old_l2_phys = old_l3_entry.addr().as_u64();
        let old_l2 = phys_to_table(old_l2_phys, offset);

        // check if this L2 has any L1 tables
        let mut l2_has_l1 = false;
        for l2_entry in old_l2.iter() {
            if l2_entry.flags().contains(PageTableFlags::PRESENT) && !l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                l2_has_l1 = true;
                break;
            }
        }

        if !l2_has_l1 {
            continue; // all 2 MiB huge pages = offset mapping, drop entire L2
        }

        // has L1 tables — rebuild this L2
        let (new_l2, new_l2_phys) = alloc_table();
        for l2_idx in 0..512usize {
            let old_l2_entry = &old_l2[l2_idx];
            if !old_l2_entry.flags().contains(PageTableFlags::PRESENT) {
                continue;
            }
            if old_l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                // 2 MiB huge page = offset mapping, drop
                continue;
            }

            // L1 table — copy it
            let old_l1_phys = old_l2_entry.addr().as_u64();
            let old_l1 = phys_to_table(old_l1_phys, offset);
            let (new_l1, new_l1_phys) = alloc_table();

            for l1_index in 0..512usize {
                let old_l1_entry = &old_l1[l1_index];
                if old_l1_entry.flags().contains(PageTableFlags::PRESENT) {
                    // copy the leaf mapping (same physical page, same flags)
                    new_l1[l1_index].set_addr(old_l1_entry.addr(), old_l1_entry.flags());
                    let leaf_physical = old_l1_entry.addr().as_u64();
                    // only track pages >= 2 MiB — pages below 2 MiB are never
                    // in our physical allocator so they can't be accidentally freed
                    if leaf_physical >= TWO_MIB {
                        kernel_pages.push(leaf_physical);
                    }
                    kept += 1;
                }
            }

            new_l2[l2_idx].set_addr(PhysAddr::new(new_l1_phys), old_l2_entry.flags());
        }

        new_l3[l3_idx].set_addr(PhysAddr::new(new_l2_phys), old_l3_entry.flags());
    }
    kept
}

// ---------------------------------------------------------------------------
// Phase B3: replace reused bootloader tables at L4[0..1]
// ---------------------------------------------------------------------------

/// The L3 at L4[0] and L2 at L4[0]/L3[0] were reused from the bootloader
/// during init(). Replace them with our own allocations.
fn replace_reused_bootloader_tables(l4_table: &mut PageTable, offset: u64) {
    // Replace L3 at L4[0]
    if l4_table[0].flags().contains(PageTableFlags::PRESENT) {
        let old_l3_phys = l4_table[0].addr().as_u64();
        // only replace if it's in bootloader memory (above 2 MiB, not a bootstrap frame)
        if old_l3_phys >= TWO_MIB {
            let old_l3 = phys_to_table(old_l3_phys, offset);
            let (new_l3, new_l3_phys) = alloc_table();

            // copy all L3 entries
            for i in 0..512usize {
                if old_l3[i].flags().contains(PageTableFlags::PRESENT) {
                    new_l3[i].set_addr(old_l3[i].addr(), old_l3[i].flags());
                }
            }

            // now replace L2 at L3[0] if it's also bootloader-owned
            if new_l3[0].flags().contains(PageTableFlags::PRESENT) && !new_l3[0].flags().contains(PageTableFlags::HUGE_PAGE) {
                let old_l2_phys = new_l3[0].addr().as_u64();
                if old_l2_phys >= TWO_MIB {
                    let old_l2 = phys_to_table(old_l2_phys, offset);
                    let (new_l2, new_l2_phys) = alloc_table();
                    for i in 0..512usize {
                        if old_l2[i].flags().contains(PageTableFlags::PRESENT) {
                            new_l2[i].set_addr(old_l2[i].addr(), old_l2[i].flags());
                        }
                    }
                    new_l3[0].set_addr(PhysAddr::new(new_l2_phys), TABLE_FLAGS);
                    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Replaced bootloader L2 at L4[0]/L3[0] ({:#x})", old_l2_phys);
                }
            }

            l4_table[0].set_addr(PhysAddr::new(new_l3_phys), TABLE_FLAGS);
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Replaced bootloader L3 at L4[0] ({:#x})", old_l3_phys);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Translate a virtual address to its physical address by walking the page tables.
/// Returns None if the address is not mapped.
pub fn virtual_to_physical(virt: u64, physical_memory_offset: u64) -> Option<u64> {
    let (l4_frame, _) = Cr3::read();
    let l4 = phys_to_table(l4_frame.start_address().as_u64(), physical_memory_offset);

    let l4_idx = ((virt >> 39) & 0x1FF) as usize;
    let l3_idx = ((virt >> 30) & 0x1FF) as usize;
    let l2_idx = ((virt >> 21) & 0x1FF) as usize;
    let l1_idx = ((virt >> 12) & 0x1FF) as usize;
    let page_offset = virt & 0xFFF;

    let l4e = &l4[l4_idx];
    if !l4e.flags().contains(PageTableFlags::PRESENT) { return None; }

    let l3 = phys_to_table(l4e.addr().as_u64(), physical_memory_offset);
    let l3e = &l3[l3_idx];
    if !l3e.flags().contains(PageTableFlags::PRESENT) { return None; }
    if l3e.flags().contains(PageTableFlags::HUGE_PAGE) {
        return Some(l3e.addr().as_u64() + (virt & (ONE_GIB - 1)));
    }

    let l2 = phys_to_table(l3e.addr().as_u64(), physical_memory_offset);
    let l2e = &l2[l2_idx];
    if !l2e.flags().contains(PageTableFlags::PRESENT) { return None; }
    if l2e.flags().contains(PageTableFlags::HUGE_PAGE) {
        return Some(l2e.addr().as_u64() + (virt & (TWO_MIB - 1)));
    }

    let l1 = phys_to_table(l2e.addr().as_u64(), physical_memory_offset);
    let l1e = &l1[l1_idx];
    if !l1e.flags().contains(PageTableFlags::PRESENT) { return None; }
    Some(l1e.addr().as_u64() + page_offset)
}

fn has_region_in_range(memory_regions: &MemoryRegions, start: u64, end: u64) -> bool {
    memory_regions.iter().any(|r| r.start < end && r.end > start)
}

fn phys_to_table(phys: u64, offset: u64) -> &'static mut PageTable {
    unsafe { &mut *((phys + offset) as *mut PageTable) }
}

fn get_or_create_table(entry: &mut PageTableEntry, bootstrap: &mut BootstrapFrameAllocator, offset: u64) -> &'static mut PageTable {
    if entry.flags().contains(PageTableFlags::PRESENT) {
        assert!(!entry.flags().contains(PageTableFlags::HUGE_PAGE), "expected table pointer, got huge page");
        return phys_to_table(entry.addr().as_u64(), offset);
    }
    let frame = bootstrap.alloc_frame();
    entry.set_addr(PhysAddr::new(frame), TABLE_FLAGS);
    phys_to_table(frame, offset)
}

// ---------------------------------------------------------------------------
// Bootstrap frame allocator (early boot only, uses frames below 2 MiB)
// ---------------------------------------------------------------------------

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
            let mut addr = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
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
        unsafe { core::ptr::write_bytes((frame + self.offset) as *mut u8, 0, PAGE_SIZE as usize) };
        frame
    }
}
