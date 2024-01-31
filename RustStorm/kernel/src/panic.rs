use core::panic::PanicInfo;
use crate::{log, log_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_println!(log::SubSystem::Kernel, log::LogLevel::Error, "KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
