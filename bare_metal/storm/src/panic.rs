use core::panic::PanicInfo;
use crate::{log, log_println, qemu};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_println!(log::SubSystem::Kernel, log::LogLevel::Error, "KERNEL PANIC: {}", info);
    qemu::exit(1); // exit code 1 → QEMU exit 3 (failure)
}
