// physical
// 00xxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx
// 0x0000_0000_0000 - 0x3fff_ffff_ffff

// kernel (virtual in physical half)
// 0x3FFF_0000_0000 - 0x3FFF_FFFF_FFFF

// virtual (user space)
// 01xxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx  xxxxxxxx xxxxxxxx
// 0x4000_0000_0000 - 0x7fff_ffff_ffff

use bootloader_api::info::{ MemoryRegions, MemoryRegionKind };
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::{structures::paging::{PhysFrame, FrameAllocator, Size4KiB, FrameDeallocator}, PhysAddr};

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
    pub static ref ALLOCATOR: Mutex<Option<PhysicalFrameAllocator>> = {
        Mutex::new(None)
    };
}

pub fn init(memory_regions: &MemoryRegions) {
    let mut allocator = ALLOCATOR.lock();
    *allocator = Some(PhysicalFrameAllocator::init(memory_regions));
}

struct FreeFrame {
    next_frame: Option<*mut FreeFrame>,
    previous_frame: Option<*mut FreeFrame>
}

unsafe impl Send for FreeFrame {}

pub struct PhysicalFrameAllocator {
    first_free: Option<*mut FreeFrame>,
    free_frame_count: u64,
    used_frame_count: u64
}

unsafe impl Send for PhysicalFrameAllocator {}

impl PhysicalFrameAllocator {
    pub fn init(memory_regions: &MemoryRegions) -> Self {
        // get usable regions from memory map
        let regions = memory_regions.iter();
        let usable_regions = regions
            .filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        let mut frame_count: u64 = 0;
        let mut previous_frame: Option<*mut FreeFrame> = None;
        let mut first_free: Option<*mut FreeFrame> = None;
        for frame in frame_addresses {
            let free_frame = frame as *mut FreeFrame;
            unsafe { (*free_frame).previous_frame = previous_frame; }
            match previous_frame {
                Some(p) => unsafe { (*p).next_frame = Some(free_frame) },
                None => first_free = Some(free_frame)
            };
            previous_frame = Some(free_frame);
            frame_count += 1;
        }
        match previous_frame {
            Some(p) => unsafe { (*p).next_frame = None },
            None => ()
        }

        PhysicalFrameAllocator { first_free: first_free, free_frame_count: frame_count, used_frame_count: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut first_free = self.first_free.unwrap();
        let mut next_frame: *mut FreeFrame;
        unsafe { next_frame = (*first_free).next_frame.unwrap(); }

        self.first_free = Some(next_frame);
        unsafe { (*next_frame).previous_frame = None; }
        self.free_frame_count -= 1;
        self.used_frame_count += 1;

        Some(PhysFrame::from_start_address(PhysAddr::new(first_free as u64)).unwrap())
    }
}

impl FrameDeallocator<Size4KiB> for PhysicalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        let free_frame = frame.start_address().as_u64() as *mut FreeFrame;
        let old_frame = self.first_free.unwrap();

        unsafe {
            (*free_frame).previous_frame = None;
            (*free_frame).next_frame = Some(old_frame);
            (*old_frame).previous_frame = Some(free_frame);
        }

        self.first_free = Some(free_frame);
        self.used_frame_count -= 1;
        self.free_frame_count += 1;
    }
}