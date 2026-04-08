//! Dual-output kernel logger (serial + framebuffer).
//!
//! Format: `[SubSystem    ] 0.123s LEVEL - message`
//!
//! Serial gets ANSI colors; framebuffer gets per-section RGB colors.
//! The framebuffer backend is optional — before `init_framebuffer()` is called
//! (or if no framebuffer is available) output goes to serial only.

use core::fmt::Write;
use core::sync::atomic::{AtomicU64, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550};

use crate::framebuffer::FramebufferWriter;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Minimum severity shown on serial.
const SERIAL_LOG_LEVEL: LogLevel = LogLevel::Debug;
/// Minimum severity shown on the framebuffer (screen space is precious).
const FB_LOG_LEVEL: LogLevel = LogLevel::Information;

// ---------------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------------

static BOOT_TSC: AtomicU64 = AtomicU64::new(0);

/// Call once at the very start of kernel_main to anchor the boot timestamp.
pub fn init_boot_tsc() {
    BOOT_TSC.store(rdtsc(), Ordering::Relaxed);
}

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Approximate seconds + milliseconds since boot.
/// Assumes ~1 GHz TSC (QEMU TCG default). Inaccurate on real hardware
/// without proper calibration, but good enough for relative ordering.
fn elapsed() -> (u64, u64) {
    let ticks = rdtsc().saturating_sub(BOOT_TSC.load(Ordering::Relaxed));
    let secs = ticks / 1_000_000_000;
    let millis = (ticks % 1_000_000_000) / 1_000_000;
    (secs, millis)
}

// ---------------------------------------------------------------------------
// Serial backend
// ---------------------------------------------------------------------------

struct Serial(Uart16550<PioBackend>);

impl core::fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.send_bytes_exact(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    static ref SERIAL: Mutex<Serial> = {
        let mut uart = unsafe { Uart16550::new_port(0x3F8).unwrap() };
        uart.init(Config::default()).expect("Failed to init serial port");
        Mutex::new(Serial(uart))
    };
}

// ---------------------------------------------------------------------------
// Framebuffer backend (optional, initialized later)
// ---------------------------------------------------------------------------

lazy_static! {
    static ref FRAMEBUFFER: Mutex<Option<FramebufferWriter>> = Mutex::new(None);
}

/// Enable framebuffer output. Called once from main after extracting the
/// framebuffer from BootInfo.
pub fn init_framebuffer(writer: FramebufferWriter) {
    *FRAMEBUFFER.lock() = Some(writer);
}

/// Remap the framebuffer to use identity mapping instead of the bootloader's
/// offset mapping. The physical address must already be identity-mapped.
pub fn remap_framebuffer(framebuffer_physical_address: u64) {
    if let Some(ref mut framebuffer) = *FRAMEBUFFER.lock() {
        framebuffer.remap_buffer(framebuffer_physical_address as *mut u8);
    }
}

// ---------------------------------------------------------------------------
// Subsystem & LogLevel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum SubSystem {
    Kernel,
    Boot,
    X86_64,
    Physical,
    KernelMemory,
}

impl core::fmt::Display for SubSystem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            SubSystem::Kernel => "Kernel",
            SubSystem::Boot => "Boot",
            SubSystem::X86_64 => "x86_64",
            SubSystem::Physical => "Physical",
            SubSystem::KernelMemory => "KernelMem",
        };
        write!(f, "{:<9}", name)
    }
}

// Pastel ANSI colors (256-color mode: 38;5;N)
fn subsystem_ansi(ss: &SubSystem) -> &'static str {
    match ss {
        SubSystem::Kernel => "38;5;210",      // pastel rose
        SubSystem::Boot => "38;5;150",         // pastel green
        SubSystem::X86_64 => "38;5;222",       // pastel gold
        SubSystem::Physical => "38;5;111",     // pastel blue
        SubSystem::KernelMemory => "38;5;183", // pastel lavender
    }
}

// Pastel RGB colors for framebuffer
fn subsystem_rgb(ss: &SubSystem) -> (u8, u8, u8) {
    match ss {
        SubSystem::Kernel => (240, 160, 160),      // pastel rose
        SubSystem::Boot => (160, 220, 160),         // pastel green
        SubSystem::X86_64 => (230, 210, 140),       // pastel gold
        SubSystem::Physical => (140, 180, 240),     // pastel blue
        SubSystem::KernelMemory => (200, 170, 240), // pastel lavender
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Information,
    Warning,
    Error,
}

fn level_label(ll: &LogLevel) -> &'static str {
    match ll {
        LogLevel::Debug => "DEBUG",
        LogLevel::Information => "INFO ",
        LogLevel::Warning => "WARN ",
        LogLevel::Error => "ERROR",
    }
}

fn level_ansi(ll: &LogLevel) -> &'static str {
    match ll {
        LogLevel::Debug => "38;5;243",       // soft gray
        LogLevel::Information => "38;5;252",  // light gray
        LogLevel::Warning => "38;5;222",      // pastel yellow
        LogLevel::Error => "38;5;210",        // pastel red
    }
}

fn level_rgb(ll: &LogLevel) -> (u8, u8, u8) {
    match ll {
        LogLevel::Debug => (140, 140, 140),       // soft gray
        LogLevel::Information => (210, 210, 210),  // light gray
        LogLevel::Warning => (240, 210, 130),      // pastel yellow
        LogLevel::Error => (240, 150, 150),        // pastel red
    }
}

// ---------------------------------------------------------------------------
// Core print function — dispatches to both backends
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub fn _print(sub_system: SubSystem, log_level: LogLevel, args: ::core::fmt::Arguments) {
    let (secs, millis) = elapsed();

    // --- serial (always available) ---
    if log_level >= SERIAL_LOG_LEVEL {
        let mut s = SERIAL.lock();
        let _ = write!(
            s,
            "\x1b[0m[\x1b[{}m{}\x1b[0m] \x1b[38;5;243m{}.{:03}s \x1b[{}m{} - ",
            subsystem_ansi(&sub_system),
            sub_system,
            secs,
            millis,
            level_ansi(&log_level),
            level_label(&log_level),
        );
        let _ = s.write_fmt(args);
        let _ = write!(s, "\x1b[0m");
    }

    // --- framebuffer (if initialized) ---
    if log_level >= FB_LOG_LEVEL {
        if let Some(ref mut framebuffer) = *FRAMEBUFFER.lock() {
            let (sr, sg, sb) = subsystem_rgb(&sub_system);
            framebuffer.set_color(sr, sg, sb);
            let _ = write!(framebuffer, "[{}] ", sub_system);

            framebuffer.set_color(150, 150, 150);
            let _ = write!(framebuffer, "{}.{:03}s ", secs, millis);

            let (lr, lg, lb) = level_rgb(&log_level);
            framebuffer.set_color(lr, lg, lb);
            let _ = write!(framebuffer, "{} - ", level_label(&log_level));

            framebuffer.set_color(lr, lg, lb);
            let _ = framebuffer.write_fmt(args);
        }
    }
}

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

/// Low-level print (no newline). Prefer `log_println!`.
#[macro_export]
macro_rules! serial_print {
    ($ss:path, $ll:path, $($arg:tt)*) => ($crate::log::_print($ss, $ll, format_args!($($arg)*)));
}

/// Log a line to all backends. Appends a trailing newline.
#[macro_export]
macro_rules! log_println {
    ($ss:path, $ll:path) => ($crate::log::_print($ss, $ll, format_args!("\n")));
    ($ss:path, $ll:path, $fmt:expr) => ($crate::log::_print($ss, $ll, format_args!(concat!($fmt, "\n"))));
    ($ss:path, $ll:path, $fmt:expr, $($arg:tt)*) => ($crate::log::_print($ss, $ll, format_args!(concat!($fmt, "\n"), $($arg)*)));
}
