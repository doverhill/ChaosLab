// use virtual address space:
// bottom half is physical memory identity mapped so that virtual addresses with top bit = 0 are physical
// upper half is virtual per process address space

// 1. create a kernel address space that only contains the bottom half identity mapped. this address space is used by all kernel threads. these page tables will also be reused by all process address spaces (these page tables)
// 2. set up irq and exception handlers
// 3. set up syscalls
// 3. start all processors
// each processor has its own process list
// 4. parse ramdisk and start a process for each provided elf image

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod address_space;
mod ap_trampoline;
mod apic;
mod framebuffer;
mod gdt;
mod interrupts;
mod kernel_memory;
mod log;
mod memory_setup;
mod page_tables;
mod panic;
mod physical_memory;
mod process;
mod qemu;
mod scheduler;
mod syscall;
mod timer;
mod virtual_memory;

use alloc::boxed::Box;
#[allow(deprecated)]
use bootloader_api::config::FrameBuffer;
use bootloader_api::{config::Mapping, config::Mappings, entry_point, BootloaderConfig};

pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;
pub const GB: usize = 1024 * 1024 * 1024;

#[allow(deprecated)]
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings = Mappings::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 128 * 1024;
    let mut framebuffer = FrameBuffer::new_default();
    framebuffer.minimum_framebuffer_width = Some(800);
    framebuffer.minimum_framebuffer_height = Some(600);
    config.frame_buffer = framebuffer;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    log::init_boot_tsc();
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Starting RustStorm kernel");

    // extract framebuffer for screen output
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        log_println!(
            log::SubSystem::Boot,
            log::LogLevel::Information,
            "Framebuffer: {}x{} {:?} {}bpp (stride {})",
            info.width,
            info.height,
            info.pixel_format,
            info.bytes_per_pixel * 8,
            info.stride
        );

        // obtain a 'static buffer reference independent of boot_info borrow
        let buffer_pointer = framebuffer.buffer_mut().as_mut_ptr();
        let len = info.byte_len;
        let buffer = unsafe { core::slice::from_raw_parts_mut(buffer_pointer, len) };

        let writer = framebuffer::FramebufferWriter::new(buffer, info);
        log_println!(
            log::SubSystem::Boot,
            log::LogLevel::Information,
            "Screen console: {}x{} chars",
            writer.cols(),
            writer.rows()
        );
        log::init_framebuffer(writer);
        log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Framebuffer logger active");
    }

    gdt::init();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "GDT loaded with kernel code segment and TSS");

    interrupts::init_exceptions();

    let physical_memory_offset = boot_info.physical_memory_offset.into_option().expect("bootloader did not provide physical memory offset");
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Physical memory offset: {:#x} (L4[{}])", physical_memory_offset, physical_memory_offset / (512 * crate::GB as u64));
    memory_setup::init_identity_mapping(physical_memory_offset, &boot_info.memory_regions);

    // Save everything we need from boot_info BEFORE physical_memory::init.
    // boot_info and memory_regions live in Bootloader-marked pages which
    // will be reclaimed after decoupling.
    let rsdp_address = boot_info.rsdp_addr;
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "RSDP address: {:?}", rsdp_address);

    // Snapshot framebuffer physical address (needed for identity mapping).
    // Walk the page tables to find the actual physical address backing the
    // framebuffer virtual pointer (the bootloader may use a separate mapping).
    let framebuffer_physical = boot_info.framebuffer.as_ref().map(|framebuffer| {
        let virtual_address = framebuffer.buffer().as_ptr() as u64;
        let size = framebuffer.info().byte_len;
        let physical_address = page_tables::virtual_to_physical(virtual_address, physical_memory_offset)
            .expect("framebuffer virtual address not mapped");
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "Framebuffer: virtual={:#x}, physical={:#x}, size={} KiB", virtual_address, physical_address, size / 1024);
        (physical_address, size)
    });

    // snapshot bootloader region ranges to the stack (memory_regions itself is in bootloader memory)
    let mut bootloader_regions = [(0u64, 0u64); 8];
    let mut bootloader_region_count = 0;
    for region in boot_info.memory_regions.iter() {
        if region.kind == bootloader_api::info::MemoryRegionKind::Bootloader && bootloader_region_count < bootloader_regions.len() {
            bootloader_regions[bootloader_region_count] = (region.start, region.end);
            bootloader_region_count += 1;
        }
    }

    // initialize frame allocator with Usable memory only
    physical_memory::init(&boot_info.memory_regions);

    // Fix null guard FIRST — the heap allocator uses virtual_memory which
    // walks page tables via identity mapping. The L4 table at 0x101000 must
    // be accessible before any heap allocation can occur.
    memory_setup::fix_null_guard(physical_memory_offset);

    // Pre-allocate L3 tables for kernel virtual memory (L4[128..256]) so that
    // all future address spaces can share the kernel half by copying L4 entries.
    memory_setup::pre_allocate_kernel_virtual_l3_tables(physical_memory_offset);

    // ---- Decouple from bootloader page tables ----
    // After this, no page table pages are in bootloader memory.
    // The framebuffer gets identity-mapped and its pointer updated.
    let kernel_leaf_pages = if let Some((physical_address, size)) = framebuffer_physical {
        memory_setup::decouple_from_bootloader(physical_memory_offset, physical_address, size)
    } else {
        memory_setup::decouple_from_bootloader(physical_memory_offset, 0, 0)
    };

    // ---- Reclaim bootloader memory ----
    physical_memory::reclaim_bootloader(&bootloader_regions[..bootloader_region_count], &kernel_leaf_pages);

    // sanity check: kernel heap allocator still works after all the page table surgery
    let _heap_test = Box::new(42u64);


    apic::init(rsdp_address);

    // spawn test kernel threads
    for i in 0..6 {
        scheduler::spawn(test_thread_function, i);
    }

    log_println!(log::SubSystem::Boot, log::LogLevel::Information,
        "Boot complete — BSP entering scheduler");

    // BSP enters the scheduler as CPU 0
    scheduler::run_on_cpu(0);
}

/// Test thread function — logs a message, yields, repeats.
fn test_thread_function(thread_number: u64) -> ! {
    loop {
        log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
            "Thread {} running", thread_number);
        // busy-wait a bit to simulate work (PM timer based)
        timer::delay_milliseconds(500);
        scheduler::yield_thread();
    }
}
