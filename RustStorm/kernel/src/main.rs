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
mod apic;
mod framebuffer;
mod gdt;
mod interrupts;
mod kernel_memory;
mod log;
mod panic;
mod physical;
mod process;
mod qemu;
mod syscall;

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
    let mut fb = FrameBuffer::new_default();
    fb.minimum_framebuffer_width = Some(800);
    fb.minimum_framebuffer_height = Some(600);
    config.frame_buffer = fb;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    log::init_boot_tsc();
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Starting RustStorm kernel");

    // extract framebuffer for screen output
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        let info = fb.info();
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
        let ptr = fb.buffer_mut().as_mut_ptr();
        let len = info.byte_len;
        let buffer = unsafe { core::slice::from_raw_parts_mut(ptr, len) };

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
    address_space::init(physical_memory_offset, &boot_info.memory_regions);

    // save everything we need from boot_info BEFORE physical::init
    // (boot_info lives in Bootloader-marked pages)
    let rsdp_addr = boot_info.rsdp_addr;
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "RSDP address: {:?}", rsdp_addr);

    // initialize frame allocator — only Usable memory goes into the free list.
    // Bootloader-marked memory (kernel code, stack, page tables) is preserved.
    // boot_info lives in Bootloader memory, so don't access it after this.
    physical::init(&boot_info.memory_regions);

    // sanity check: kernel heap allocator works (Box triggers GlobalAlloc)
    let _heap_test = Box::new(42u64);

    apic::init(rsdp_addr);

    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Boot complete — press any key or wait 20s");
    qemu::wait_or_keypress(20);
    qemu::exit(0);
}
