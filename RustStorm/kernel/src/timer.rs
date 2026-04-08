//! Hardware timer utilities.
//!
//! Provides spin-wait delay functions using the ACPI PM Timer (3,579,545 Hz).
//! The PM timer port depends on the chipset:
//! - 0xB008 on i440fx/PIIX3 (QEMU default)
//! - 0x608 on Q35/ICH9
//!
//! The PM timer is a 24-bit counter on PIIX3, wrapping every ~4.69 seconds.

const PM_TIMER_PORT: u16 = 0xB008;
const PM_TIMER_FREQ: u64 = 3_579_545;
const PM_TIMER_MASK: u32 = 0x00FF_FFFF;

#[inline]
pub fn pm_timer_read() -> u32 {
    (unsafe { x86_64::instructions::port::Port::<u32>::new(PM_TIMER_PORT).read() }) & PM_TIMER_MASK
}

/// Spin-wait for the specified number of microseconds using the PM Timer.
pub fn delay_microseconds(microseconds: u64) {
    let target_ticks = microseconds * PM_TIMER_FREQ / 1_000_000;
    let start = pm_timer_read();
    while ((pm_timer_read().wrapping_sub(start)) & PM_TIMER_MASK) as u64 <= target_ticks {
        core::hint::spin_loop();
    }
}

/// Spin-wait for the specified number of milliseconds using the PM Timer.
pub fn delay_milliseconds(milliseconds: u64) {
    // for delays > 4s, loop in smaller chunks to handle 24-bit wrap
    let mut remaining_microseconds = milliseconds * 1000;
    while remaining_microseconds > 0 {
        let chunk = remaining_microseconds.min(3_000_000); // max ~3s per chunk
        delay_microseconds(chunk);
        remaining_microseconds -= chunk;
    }
}
