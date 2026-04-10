//! Hardware timer utilities.
//!
//! Provides:
//! - PM timer spin-wait delays (ACPI PM Timer, 3,579,545 Hz)
//! - TSC frequency calibration (using PM timer as reference)
//! - APIC timer one-shot arming/disarming for preemption and timed wakeups
//!
//! The PM timer port depends on the chipset:
//! - 0xB008 on i440fx/PIIX3 (QEMU default)
//! - 0x608 on Q35/ICH9

use core::sync::atomic::{AtomicU64, Ordering};
use crate::{log, log_println};

const PM_TIMER_PORT: u16 = 0xB008;
const PM_TIMER_FREQ: u64 = 3_579_545;
const PM_TIMER_MASK: u32 = 0x00FF_FFFF;

// APIC timer registers (offsets from APIC base 0xFEE00000)
const APIC_BASE: u64 = 0xFEE00000;
const APIC_TIMER_LVT: u64 = APIC_BASE + 0x320;     // LVT Timer Register
const APIC_TIMER_INITIAL: u64 = APIC_BASE + 0x380;  // Initial Count Register
const APIC_TIMER_CURRENT: u64 = APIC_BASE + 0x390;  // Current Count Register
const APIC_TIMER_DIVIDE: u64 = APIC_BASE + 0x3E0;   // Divide Configuration Register

/// APIC timer vector (must match interrupts.rs)
const TIMER_VECTOR: u32 = 0xFE;

/// TSC ticks per second (calibrated at boot).
static TSC_FREQUENCY: AtomicU64 = AtomicU64::new(0);
/// APIC timer ticks per second (calibrated at boot).
static APIC_TIMER_FREQUENCY: AtomicU64 = AtomicU64::new(0);

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

// ---------------------------------------------------------------------------
// TSC and APIC timer calibration
// ---------------------------------------------------------------------------

/// Read the TSC (Time Stamp Counter).
#[inline(always)]
pub fn read_tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Get the calibrated TSC frequency in Hz.
pub fn tsc_frequency() -> u64 {
    TSC_FREQUENCY.load(Ordering::Relaxed)
}

/// Convert milliseconds to TSC ticks.
pub fn milliseconds_to_ticks(milliseconds: u64) -> u64 {
    milliseconds * tsc_frequency() / 1000
}

/// Calibrate the TSC and APIC timer frequencies using the PM timer as a
/// reference. Call once on the BSP during boot, after the APIC is enabled.
pub fn calibrate() {
    // --- Calibrate TSC ---
    let calibration_microseconds: u64 = 10_000; // 10ms
    let tsc_start = read_tsc();
    delay_microseconds(calibration_microseconds);
    let tsc_end = read_tsc();
    let tsc_delta = tsc_end - tsc_start;
    let tsc_freq = tsc_delta * 1_000_000 / calibration_microseconds;
    TSC_FREQUENCY.store(tsc_freq, Ordering::Release);

    // --- Calibrate APIC timer ---
    // Set divide-by-1 for maximum resolution
    unsafe { core::ptr::write_volatile(APIC_TIMER_DIVIDE as *mut u32, 0x0B) }; // divide by 1

    // Set a large initial count and measure how much counts down in 10ms
    unsafe { core::ptr::write_volatile(APIC_TIMER_INITIAL as *mut u32, 0xFFFF_FFFF) };
    delay_microseconds(calibration_microseconds);
    let apic_remaining = unsafe { core::ptr::read_volatile(APIC_TIMER_CURRENT as *const u32) };
    let apic_delta = 0xFFFF_FFFFu32 - apic_remaining;
    let apic_freq = (apic_delta as u64) * 1_000_000 / calibration_microseconds;
    APIC_TIMER_FREQUENCY.store(apic_freq, Ordering::Release);

    // Stop the timer
    unsafe { core::ptr::write_volatile(APIC_TIMER_INITIAL as *mut u32, 0) };

    log_println!(log::SubSystem::X86_64, log::LogLevel::Information,
        "Timer calibration: TSC={} MHz, APIC timer={} MHz",
        tsc_freq / 1_000_000, apic_freq / 1_000_000);
}

// ---------------------------------------------------------------------------
// APIC timer one-shot arming/disarming
// ---------------------------------------------------------------------------

/// Arm the APIC timer in one-shot mode to fire after `milliseconds`.
/// The timer fires vector 0xFE and then stops (one-shot).
pub fn arm_apic_timer_milliseconds(milliseconds: u64) {
    let apic_freq = APIC_TIMER_FREQUENCY.load(Ordering::Relaxed);
    if apic_freq == 0 { return; } // not calibrated yet
    let ticks = (milliseconds * apic_freq / 1000) as u32;
    arm_apic_timer_ticks(ticks);
}

/// Arm the APIC timer in one-shot mode with a raw tick count.
pub fn arm_apic_timer_ticks(ticks: u32) {
    if ticks == 0 { return; }
    unsafe {
        // LVT Timer: one-shot mode (bits 17:18 = 00), vector = TIMER_VECTOR
        core::ptr::write_volatile(APIC_TIMER_LVT as *mut u32, TIMER_VECTOR);
        // Divide by 1
        core::ptr::write_volatile(APIC_TIMER_DIVIDE as *mut u32, 0x0B);
        // Initial count — starts the countdown
        core::ptr::write_volatile(APIC_TIMER_INITIAL as *mut u32, ticks);
    }
}

/// Disarm the APIC timer (stop it from firing).
pub fn disarm_apic_timer() {
    unsafe {
        // Writing 0 to initial count stops the timer
        core::ptr::write_volatile(APIC_TIMER_INITIAL as *mut u32, 0);
    }
}
