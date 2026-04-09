//! QEMU integration: debug exit and interactive wait.

use crate::{log, log_println, timer};

/// Spin-wait for `seconds` or until a PS/2 keypress, whichever comes first.
pub fn wait_or_keypress(seconds: u64) {
    let total_ticks = seconds * 3_579_545; // PM_TIMER_FREQ
    let mut remaining = total_ticks;
    let mut start = timer::pm_timer_read();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
        "PM Timer: port=0xb008, start={}, waiting {}s", start, seconds);

    let mut last_report_second: u64 = 0;
    loop {
        let now = timer::pm_timer_read();
        let elapsed = (now.wrapping_sub(start)) & 0x00FF_FFFF;

        if elapsed as u64 >= remaining {
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "PM Timer: timeout reached");
            break;
        }

        // reset baseline well before 24-bit wrap
        if elapsed > 0x00C0_0000 {
            remaining -= elapsed as u64;
            start = now;
        }

        let elapsed_total = total_ticks - remaining + elapsed as u64;
        let elapsed_second = elapsed_total / 3_579_545;
        if elapsed_second > last_report_second && elapsed_second % 5 == 0 {
            last_report_second = elapsed_second;
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
                "PM Timer: ~{}s elapsed", elapsed_second);
        }

        // PS/2 keyboard check
        let status: u8 = unsafe { x86_64::instructions::port::Port::new(0x64).read() };
        if status & 1 != 0 {
            let _scancode: u8 = unsafe { x86_64::instructions::port::Port::new(0x60).read() };
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Keypress detected");
            break;
        }

        core::hint::spin_loop();
    }
}

/// Exit QEMU via the isa-debug-exit device (port 0xf4, 1-byte write).
/// QEMU exit status = (value << 1) | 1, so 0 → exit 1, 1 → exit 3.
pub fn exit(code: u8) -> ! {
    unsafe { x86_64::instructions::port::Port::<u8>::new(0xf4).write(code) };
    loop { x86_64::instructions::hlt(); }
}
