use core::num;

use crate::{log, log_println, physical};
use alloc::alloc::{GlobalAlloc, Layout};
use lazy_static::lazy_static;
use spin::Mutex;
use derivative::Derivative;

#[global_allocator]
pub static ALLOCATOR: KernelMemoryAllocator = KernelMemoryAllocator;

pub struct KernelMemoryAllocator;

unsafe impl GlobalAlloc for KernelMemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        INNER.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        INNER.lock().dealloc(ptr, layout);
    }
}

lazy_static! {
    static ref INNER: Mutex<InnerKernelAllocator> = Mutex::new(InnerKernelAllocator::new());
}

#[derive(Debug)]
struct AllocatorBlockChain {
    pub slot_size: usize,
    pub block_count: usize,
    pub total_slot_count: usize,
    pub total_free_slot_count: usize,
    pub first_block: Option<*mut AllocatorBlock>,
}

unsafe impl Send for AllocatorBlockChain {}

#[repr(C, packed)]
#[derive(Derivative)]
#[derivative(Debug)]
struct AllocatorBlock {
    #[derivative(Debug="ignore")]
    data: [u8; 4040],
    pub slot_size: usize,
    pub slot_count: usize,
    pub free_slot_count: usize,
    pub first_free_slot: Option<*mut u8>,
    pub next_block: Option<*mut AllocatorBlock>,
}

unsafe impl Send for AllocatorBlock {}

struct InnerKernelAllocator {
    // optional pointers to supported sizes:
    // 4, 8, 16, 32, 64, 128, 256, 512, 1024 = 9 sizes
    block_chains: [AllocatorBlockChain; 9],
    allocated_bytes: usize,
    used_bytes: usize,
}

unsafe impl Send for InnerKernelAllocator {}

impl InnerKernelAllocator {
    pub fn new() -> Self {
        let block_chains: [AllocatorBlockChain; 9] = core::array::from_fn(|size_index| AllocatorBlockChain {
            slot_size: 4 << size_index,
            block_count: 0,
            total_slot_count: 0,
            total_free_slot_count: 0,
            first_block: None,
        });

        InnerKernelAllocator {
            block_chains: block_chains,
            allocated_bytes: 0,
            used_bytes: 0,
        }
    }

    unsafe fn allocate_new_block(size_index: usize) -> *mut AllocatorBlock {
        let block_pointer = physical::allocate(1).unwrap() as *mut AllocatorBlock;
        let block = block_pointer.as_mut().unwrap();

        block.slot_size = 4 << size_index;
        block.slot_count = 4040 / block.slot_size;
        block.free_slot_count = block.slot_count;
        block.first_free_slot = Some(block_pointer as *mut u8);
        block.next_block = None;

        // initialize next free for each slot
        let mut slot_pointer = block_pointer as *mut i32;
        let mut next_slot_index = 1;
        (0..block.slot_count).for_each(|_| {
            *slot_pointer = next_slot_index;
            slot_pointer = (slot_pointer as usize + block.slot_size) as *mut i32;
            next_slot_index += 1;
        });

        block_pointer
    }

    fn get_size_index(layout: Layout) -> (bool, usize, usize) {
        if layout.size() > 1024 {
            let number_of_pages = ((layout.size() - 1) / 4096) + 1;
            return (true, number_of_pages, 0);
        }

        let mut size = layout.size().max(layout.align()).next_power_of_two();
        if size < 4 {
            size = 4;
        }

        let size_index = size.trailing_zeros() - 2;

        (false, size, size_index as usize)
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Allocating {:?}", layout);

        let (use_own_page, size, size_index) = InnerKernelAllocator::get_size_index(layout);

        if use_own_page {
            log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Allocating block of size {}, using {} page(s)", size * 4096, size);

            // allocate physical pages for this memory
            self.allocated_bytes += layout.size();
            self.used_bytes += size * 4096;
            physical::allocate(size).unwrap()
        } else {
            log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Allocating block of size {}, using size index {}", size, size_index);

            let block_chain = &mut self.block_chains[size_index];
            assert!(block_chain.slot_size == size);

            log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Got block chain for size_index {}: {:?}", size_index, block_chain);

            // do we have any free slots in this block chain or do we need to allocate a new block?
            if block_chain.total_free_slot_count == 0 {
                // allocate a new block and put it at the start of the chain
                let new_block_pointer = InnerKernelAllocator::allocate_new_block(size_index);
                let new_block = new_block_pointer.as_mut().unwrap();

                // link
                let old_start = block_chain.first_block;
                block_chain.first_block = Some(new_block_pointer);
                new_block.next_block = old_start;

                // update block chain
                block_chain.total_slot_count += new_block.slot_count;
                block_chain.total_free_slot_count += new_block.slot_count;
                block_chain.block_count += 1;

                log_println!(
                    log::SubSystem::KernelMemory,
                    log::LogLevel::Debug,
                    "Added new block to block chain for size_index {}: {:?}",
                    size_index,
                    new_block
                );
            }

            // we now have free slots in the chain, find first free
            let block = block_chain.first_block.unwrap().as_mut().unwrap();
            loop {
                assert!(block.slot_size == size);

                if block.free_slot_count > 0 {
                    let slot_pointer = block.first_free_slot.expect("Expected block to contain free slots");

                    // update block's first_free
                    let next_free_slot_index = *(slot_pointer as *const i32);

                    return slot_pointer;
                }
                let block = block.next_block.expect("Expected block with free slots").as_mut().unwrap();
            }
        }
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Freeing {:p} {:?}", ptr, layout);

        let (use_own_page, size, size_index) = InnerKernelAllocator::get_size_index(layout);

        if use_own_page {
            log_println!(log::SubSystem::KernelMemory, log::LogLevel::Debug, "Freeing block of size {}, using {} page(s)", size * 4096, size);

            self.allocated_bytes -= layout.size();
            self.used_bytes -= size * 4096;
            physical::free(ptr, size);
        } else {
        }
    }
}
