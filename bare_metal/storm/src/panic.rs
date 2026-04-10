use core::panic::PanicInfo;
use crate::{arch, log, log_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_println!(log::SubSystem::Kernel, log::LogLevel::Error, "KERNEL PANIC: {}", info);
    arch::exit_emulator(1);
}
