// physical
// 00xxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx
// 0x0000_0000_0000 - 0x3fff_ffff_ffff

// kernel (virtual in physical half)
// 0x3FFF_0000_0000 - 0x3FFF_FFFF_FFFF

// virtual (user space)
// 01xxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx
// 0x4000_0000_0000 - 0x7fff_ffff_ffff

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
    PhysAddr,
};

use crate::{log, log_println};

lazy_static! {
    static ref ALLOCATOR: Mutex<Option<PhysicalFrameAllocator>> = Mutex::new(None);
}

pub fn init(memory_regions: &MemoryRegions) {
    log_println!(log::SubSystem::Physical, log::LogLevel::Information, "Initializing physical memory allocator");

    let mut allocator = ALLOCATOR.lock();
    *allocator = Some(PhysicalFrameAllocator::init(memory_regions));
}

pub fn allocate(number_of_pages: usize) -> Option<*mut u8> {
    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "Allocating {} pages", number_of_pages);

    assert!(number_of_pages == 1);

    let mut allocator = ALLOCATOR.lock();
    match allocator.as_mut() {
        Some(inner) => inner.allocate_frame().map(|f| f.start_address().as_u64() as *mut u8),
        None => None,
    }
}

pub fn free(page_address: *mut u8, number_of_pages: usize) {
    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "Freeing {:p} - {} pages", page_address, number_of_pages);

    assert!(number_of_pages == 1);

    let mut allocator = ALLOCATOR.lock();
    match allocator.as_mut() {
        Some(inner) => unsafe { inner.deallocate_frame(PhysFrame::containing_address(PhysAddr::new(page_address as u64))) },
        None => {}
    };
}

const FREE_FRAME_MAGIC: u64 = 0xC0CA_C07A_DEAD_BEAF;
pub const PAGE_SIZE: usize = 0x1000;

struct FreeFrame {
    magic: u64,
    next_frame: Option<*mut FreeFrame>,
    // previous_frame: Option<*mut FreeFrame>,
}

unsafe impl Send for FreeFrame {}

struct PhysicalFrameAllocator {
    first_free: Option<*mut FreeFrame>,
    free_frame_count: usize,
    used_frame_count: usize,
}

unsafe impl Send for PhysicalFrameAllocator {}

impl PhysicalFrameAllocator {
    pub fn init(memory_regions: &MemoryRegions) -> Self {
        // find the end of actual RAM (highest Usable or Bootloader region)
        let mut ram_end: u64 = 0;
        for region in memory_regions.iter() {
            match region.kind {
                MemoryRegionKind::Usable | MemoryRegionKind::Bootloader => {
                    if region.end > ram_end {
                        ram_end = region.end;
                    }
                }
                _ => {}
            }
        }

        // log memory map, separating RAM regions from MMIO
        let mut usable_bytes: u64 = 0;
        let mut bootloader_bytes: u64 = 0;
        let mut firmware_bytes: u64 = 0;
        for region in memory_regions.iter() {
            let size = region.end - region.start;
            if region.start >= ram_end {
                // high MMIO — log but don't count toward RAM totals
                log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  MMIO:     {:#012x}-{:#012x} ({} MiB) {:?}", region.start, region.end, size / (1024 * 1024), region.kind);
                continue;
            }
            match region.kind {
                MemoryRegionKind::Usable => {
                    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  Usable:   {:#012x}-{:#012x} ({} KiB)", region.start, region.end, size / 1024);
                    usable_bytes += size;
                }
                MemoryRegionKind::Bootloader => {
                    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  Reclaim:  {:#012x}-{:#012x} ({} KiB)", region.start, region.end, size / 1024);
                    bootloader_bytes += size;
                }
                kind => {
                    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  Firmware: {:#012x}-{:#012x} ({} KiB) {:?}", region.start, region.end, size / 1024, kind);
                    firmware_bytes += size;
                }
            }
        }
        let total_ram = usable_bytes + bootloader_bytes + firmware_bytes;
        log_println!(
            log::SubSystem::Physical,
            log::LogLevel::Information,
            "RAM: {} MiB total ({} MiB usable, {} MiB bootloader reserved, {} MiB firmware)",
            total_ram / crate::MB as u64,
            usable_bytes / crate::MB as u64,
            bootloader_bytes / crate::MB as u64,
            firmware_bytes / crate::MB as u64
        );

        // Pass 1: add Usable frames (safe — boot_info lives in Bootloader memory, not Usable)
        log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "Pass 1: adding usable frames to free list");
        let mut frame_count: usize = 0;
        let mut previous_frame: Option<*mut FreeFrame> = None;
        let mut first_free: Option<*mut FreeFrame> = None;
        for region in memory_regions.iter() {
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }
            Self::add_region_frames(region.start, region.end, &mut first_free, &mut previous_frame, &mut frame_count);
        }
        let usable_frames = frame_count;
        log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "Pass 1 complete: {} usable frames", usable_frames);

        // Bootloader-marked memory is NOT reclaimable yet. It contains the
        // running kernel's code, data, stack, and active page tables (L4[2+]
        // subtree, plus reused L3/L2 tables at L4[0..1]).
        //
        // To reclaim later (two-pass approach):
        //   1. Allocate fresh pages from the Usable free list
        //   2. Copy kernel segments and create new page tables
        //   3. Switch CR3 to new page tables
        //   4. Walk old page tables to identify all referenced physical pages
        //   5. Add unreferenced Bootloader pages to the free list
        log_println!(log::SubSystem::Physical, log::LogLevel::Information, "Bootloader memory: {} KiB reserved (kernel code, stack, page tables)", bootloader_bytes / 1024);

        log_println!(
            log::SubSystem::Physical,
            log::LogLevel::Information,
            "Allocator ready: {} free pages ({} MiB)",
            frame_count,
            frame_count * PAGE_SIZE / crate::MB,
        );

        PhysicalFrameAllocator {
            first_free,
            free_frame_count: frame_count,
            used_frame_count: 0,
        }
    }
    /// Add all frames in [start, end) that are >= 2 MiB to the free list.
    /// Returns the number of frames added.
    fn add_region_frames(
        start: u64,
        end: u64,
        first_free: &mut Option<*mut FreeFrame>,
        previous_frame: &mut Option<*mut FreeFrame>,
        frame_count: &mut usize,
    ) -> usize {
        let mut added = 0;
        let mut addr = start;
        while addr + PAGE_SIZE as u64 <= end {
            if addr >= 0x20_0000 {
                let free_frame = addr as *mut FreeFrame;
                unsafe {
                    (*free_frame).magic = FREE_FRAME_MAGIC;
                    (*free_frame).next_frame = None;
                }
                match *previous_frame {
                    Some(p) => unsafe { (*p).next_frame = Some(free_frame) },
                    None => *first_free = Some(free_frame),
                };
                *previous_frame = Some(free_frame);
                *frame_count += 1;
                added += 1;
            }
            addr += PAGE_SIZE as u64;
        }
        added
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let first_free = self.first_free.expect("Could not allocate physical page. Out of memory.");

        unsafe {
            if !(*first_free).magic == FREE_FRAME_MAGIC {
                panic!("Memory corruption");
            }
        }

        let next_frame = unsafe {
            (*first_free).next_frame
        };

        self.first_free = next_frame;
        self.free_frame_count -= 1;
        self.used_frame_count += 1;

        Some(PhysFrame::from_start_address(PhysAddr::new(first_free as u64)).unwrap())
    }
}

impl FrameDeallocator<Size4KiB> for PhysicalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        assert!(frame.start_address().as_u64().trailing_zeros() >= 12);

        let free_frame = frame.start_address().as_u64() as *mut FreeFrame;

        unsafe {
            (*free_frame).magic = FREE_FRAME_MAGIC;
            (*free_frame).next_frame = self.first_free;
        }

        self.first_free = Some(free_frame);
        self.used_frame_count -= 1;
        self.free_frame_count += 1;
    }
}
