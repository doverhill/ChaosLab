#![no_std]
#![no_main]

mod process;
mod serial;
mod syscall;
mod panic;

use x2apic::*;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    serial_println!("Hello from kernel");

    // clear screen
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x90;
        }
    }

    unsafe {
        match lapic::LocalApicBuilder::new().set_xapic_base(lapic::xapic_base()).build() {
            Ok(apic) => serial_println!("is bsp: {}", apic.is_bsp()),
            Err(error) => serial_println!("failed to initalize lapic: {}", error)
        }
    }

    // if let Some(_ramdisk_address) = boot_info.ramdisk_addr {

    // }

    loop {}
}
