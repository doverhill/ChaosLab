//! QEMU integration: debug exit and spin-wait using the ACPI PM Timer.
//!
//! The PM Timer runs at exactly 3,579,545 Hz (ACPI spec). On PIIX3 (i440fx)
//! it's a 24-bit counter at port 0xB008 that wraps every ~4.69s. On Q35 (ICH9)
//! it's 32-bit at port 0x608. We handle the 24-bit case by resetting the
//! baseline before each wrap.

use crate::{log, log_println};

const PM_TIMER_PORT: u16 = 0xB008; // i440fx/PIIX3; change to 0x608 for Q35
const PM_TIMER_FREQ: u64 = 3_579_545;
const PM_TIMER_MASK: u32 = 0x00FF_FFFF; // 24-bit

#[inline]
fn pm_timer_read() -> u32 {
    (unsafe { x86_64::instructions::port::Port::<u32>::new(PM_TIMER_PORT).read() }) & PM_TIMER_MASK
}

/// Read the PS/2 keyboard status port. Returns true if a key is available.
#[inline]
fn ps2_key_available() -> bool {
    let status: u8 = unsafe { x86_64::instructions::port::Port::new(0x64).read() };
    status & 1 != 0
}

/// Drain one scancode from the PS/2 data port.
#[inline]
fn ps2_read_scancode() -> u8 {
    unsafe { x86_64::instructions::port::Port::new(0x60).read() }
}

/// Spin-wait for `secs` seconds or until a PS/2 keypress, whichever comes first.
/// Uses the ACPI PM Timer (24-bit, wraps every ~4.69s).
pub fn wait_or_keypress(secs: u64) {
    let total_ticks = secs * PM_TIMER_FREQ;
    let mut remaining = total_ticks;
    let mut start = pm_timer_read();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "PM Timer: port={:#x}, start={}, waiting {}s", PM_TIMER_PORT, start, secs);

    let mut last_report_sec: u64 = 0;
    loop {
        let now = pm_timer_read();
        let elapsed = (now.wrapping_sub(start)) & PM_TIMER_MASK;

        if elapsed as u64 >= remaining {
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "PM Timer: timeout reached");
            break;
        }

        // reset baseline well before 24-bit wrap (~3.5s of the 4.69s period)
        if elapsed > 0x00C0_0000 {
            remaining -= elapsed as u64;
            start = now;
        }

        let elapsed_total = total_ticks - remaining + elapsed as u64;
        let elapsed_sec = elapsed_total / PM_TIMER_FREQ;
        if elapsed_sec > last_report_sec && elapsed_sec % 5 == 0 {
            last_report_sec = elapsed_sec;
            log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "PM Timer: ~{}s elapsed", elapsed_sec);
        }

        if ps2_key_available() {
            let _ = ps2_read_scancode();
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
