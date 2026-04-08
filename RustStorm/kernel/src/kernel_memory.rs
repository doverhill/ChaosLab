//! Kernel heap allocator backed by talc with virtual memory auto-growth.
//!
//! Uses talc's segregated-fit allocator with a custom [`Source`] implementation
//! that requests contiguous virtual pages from [`virtual_memory`] when the heap
//! needs to grow.

use core::alloc::Layout;

use talc::base::Talc;
use talc::base::binning::Binning;
use talc::source::Source;

use crate::virtual_memory;

/// Minimum number of pages to request when the heap needs to grow.
/// Requesting more at once amortizes page table overhead.
const MIN_GROWTH_PAGES: usize = 16; // 64 KiB

/// Source implementation that acquires memory from the kernel virtual memory module.
#[derive(Debug)]
pub struct VirtualMemorySource;

// SAFETY: acquire does not interact with the parent TalcLock.
// It calls virtual_memory::allocate_contiguous_pages which uses the physical
// allocator and page tables directly, not the heap allocator.
unsafe impl Source for VirtualMemorySource {
    fn acquire<B: Binning>(talc: &mut Talc<Self, B>, layout: Layout) -> Result<(), ()> {
        // calculate how many pages we need (at least MIN_GROWTH_PAGES)
        let needed_bytes = layout.size().max(layout.align());
        let needed_pages = ((needed_bytes + 0xFFF) / 0x1000).max(MIN_GROWTH_PAGES);

        let base = virtual_memory::allocate_contiguous_pages(needed_pages).ok_or(())?;
        let size = needed_pages * 0x1000;

        // claim this region for talc (returns None if the region is too small)
        unsafe {
            if talc.claim(base, size).is_none() {
                return Err(());
            }
        }

        Ok(())
    }
}

#[global_allocator]
static ALLOCATOR: talc::TalcLock<spin::Mutex<()>, VirtualMemorySource> =
    talc::TalcLock::new(VirtualMemorySource);
