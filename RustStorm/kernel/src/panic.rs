use core::panic::PanicInfo;
use crate::serial_println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("kernel panic: {}", info);
    loop {}
}
