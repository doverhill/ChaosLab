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

// pub fn init() {
//     let mut usable: u64 = 0;
//     let mut total: u64 = 0;
//     for region in boot_info.memory_regions.iter() {
//         serial_println!("Memory region {:?}", region);
//         if region.kind == MemoryRegionKind::Usable {
//             usable += region.end - region.start;
//         }
//         total += region.end - region.start;
//     }
//     serial_println!("{} of {} free", usable, total);
// }

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
        // log the full memory map so we can verify UEFI/bootloader regions are excluded
        let mut usable_bytes: u64 = 0;
        let mut reserved_bytes: u64 = 0;
        for region in memory_regions.iter() {
            let size = region.end - region.start;
            match region.kind {
                MemoryRegionKind::Usable => {
                    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  Usable:   {:#010x}-{:#010x} ({} KiB)", region.start, region.end, size / 1024);
                    usable_bytes += size;
                }
                kind => {
                    log_println!(log::SubSystem::Physical, log::LogLevel::Debug, "  Reserved: {:#010x}-{:#010x} ({} KiB) {:?}", region.start, region.end, size / 1024, kind);
                    reserved_bytes += size;
                }
            }
        }
        log_println!(log::SubSystem::Physical, log::LogLevel::Information, "Memory: {} MiB usable, {} MiB reserved", usable_bytes / crate::MB as u64, reserved_bytes / crate::MB as u64);

        // only add frames from Usable regions, skip everything below 2 MiB
        let frame_addresses = memory_regions.iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(PAGE_SIZE));

        let mut frame_count: usize = 0;
        let mut previous_frame: Option<*mut FreeFrame> = None;
        let mut first_free: Option<*mut FreeFrame> = None;
        for frame in frame_addresses {
            // skip everything below 2 MiB — used by bootloader page tables and bootstrap allocator
            if frame >= 0x20_0000 {
                let free_frame = frame as *mut FreeFrame;
                unsafe {
                    (*free_frame).magic = FREE_FRAME_MAGIC;
                    (*free_frame).next_frame = None;
                }
                match previous_frame {
                    Some(p) => unsafe { (*p).next_frame = Some(free_frame) },
                    None => first_free = Some(free_frame),
                };
                previous_frame = Some(free_frame);
                frame_count += 1;
            }
        }

        log_println!(log::SubSystem::Physical, log::LogLevel::Information, "Free pages: {} ({} MiB)", frame_count, frame_count * PAGE_SIZE / crate::MB);

        PhysicalFrameAllocator {
            first_free,
            free_frame_count: frame_count,
            used_frame_count: 0,
        }
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
        assert!(frame.start_address().as_u64().trailing_zeros() == 12);

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
