#![no_std]
#![no_main]

use library_chaos::{self, EmitType};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    library_chaos::process_emit(EmitType::Information, "Hello from userspace!");
    library_chaos::process_emit(EmitType::Debug, "Chaos test app running on Storm kernel");
    library_chaos::process_exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    library_chaos::process_emit(EmitType::Error, "PANIC in user process");
    library_chaos::process_exit(1);
}
