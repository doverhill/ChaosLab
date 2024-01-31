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

mod log;
mod address_space;
mod apic;
mod gdt;
mod interrupts;
mod kernel_memory;
mod panic;
mod physical;
mod process;
mod syscall;

use alloc::boxed::Box;
use bootloader_api::{config::Mapping, config::Mappings, entry_point, BootloaderConfig};

pub const KB: usize = 1024;
pub const MB: usize = (1024 * 1024);
pub const GB: usize = (1024 * 1024 * 1024);

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings = Mappings::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0));
    config.kernel_stack_size = 128 * 1024;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Starting RustStorm kernel");

    // let cpuid = CpuId::new();
    // let flags = cpuid.get_extended_processor_and_feature_identifiers().unwrap();
    // serial_println!("has 1GB pages: {}", flags.has_1gib_pages());

    // clear screen
    // if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
    //     for byte in framebuffer.buffer_mut() {
    //         *byte = 0x90;
    //     }
    // }

    gdt::init();

    // enable exception handling
    interrupts::init_exceptions();

    // initialize frame allocator
    // NOTE: no code can allocate before this has been run
    physical::init(&boot_info.memory_regions);

    let page = physical::allocate(1).unwrap();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Allocated frame {:p}", page);
    physical::free(page, 1);

    let _test_alloction = Box::new(1337 as u128);

    // get processors and start APs
    apic::init(boot_info.rsdp_addr);

    // if let Some(_ramdisk_address) = boot_info.ramdisk_addr {

    // }

    loop {}
}
