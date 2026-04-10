//! Dual-output kernel logger (serial + framebuffer).
//!
//! Format: `[SubSystem] 0.123s LEVEL - message`
//!
//! Serial output is synchronous (behind a spinlock — fast, useful for
//! immediate debug output). Framebuffer output is asynchronous: log calls
//! push formatted entries into a lock-free ring buffer, and a dedicated
//! `log_sink` kernel task drains the buffer and renders to the framebuffer.
//! This eliminates deadlock from preemption while holding the framebuffer lock.

use core::fmt::Write;
use core::sync::atomic::{AtomicUsize, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550};

use crate::framebuffer::FramebufferWriter;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Minimum severity shown on serial. Change to Debug when troubleshooting.
const SERIAL_LOG_LEVEL: LogLevel = LogLevel::Information;
/// Minimum severity shown on the framebuffer.
const FB_LOG_LEVEL: LogLevel = LogLevel::Information;

// ---------------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------------

static BOOT_TSC: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Call once at the very start of kernel_main to anchor the boot timestamp.
pub fn init_boot_tsc() {
    BOOT_TSC.store(rdtsc(), core::sync::atomic::Ordering::Relaxed);
}

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

fn elapsed() -> (u64, u64) {
    let ticks = rdtsc().saturating_sub(BOOT_TSC.load(core::sync::atomic::Ordering::Relaxed));
    let secs = ticks / 1_000_000_000;
    let millis = (ticks % 1_000_000_000) / 1_000_000;
    (secs, millis)
}

// ---------------------------------------------------------------------------
// Serial backend (synchronous, always available)
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
// Framebuffer backend (async via ring buffer + sink task)
// ---------------------------------------------------------------------------

lazy_static! {
    static ref FRAMEBUFFER: Mutex<Option<FramebufferWriter>> = Mutex::new(None);
}

pub fn init_framebuffer(writer: FramebufferWriter) {
    *FRAMEBUFFER.lock() = Some(writer);
}

pub fn remap_framebuffer(framebuffer_physical_address: u64) {
    if let Some(ref mut framebuffer) = *FRAMEBUFFER.lock() {
        framebuffer.remap_buffer(framebuffer_physical_address as *mut u8);
    }
}

/// A pre-formatted log entry with a fixed-size buffer (no heap allocation
/// in the log path, which could deadlock if the allocator lock is held
/// when preempted).
struct LogEntry {
    sub_system: SubSystem,
    log_level: LogLevel,
    length: usize,
    buffer: [u8; 256],
}

/// Queue of pending log entries waiting to be written to the framebuffer.
/// Protected by a spinlock (brief hold time — just push one entry).
/// The Vec itself grows on the heap but only when capacity is exceeded.
static LOG_QUEUE: Mutex<alloc::vec::Vec<LogEntry>> = Mutex::new(alloc::vec::Vec::new());

/// Write directly to framebuffer (used during early boot before scheduler).
fn write_framebuffer_sync(sub_system: SubSystem, log_level: LogLevel, secs: u64, millis: u64, args: core::fmt::Arguments) {
    if let Some(ref mut framebuffer) = *FRAMEBUFFER.lock() {
        let (sr, sg, sb) = subsystem_rgb(&sub_system);
        framebuffer.set_color(sr, sg, sb);
        let _ = write!(framebuffer, "[{}] ", sub_system);

        framebuffer.set_color(150, 150, 150);
        let _ = write!(framebuffer, "{}.{:03}s ", secs, millis);

        let (lr, lg, lb) = level_rgb(&log_level);
        framebuffer.set_color(lr, lg, lb);
        let _ = write!(framebuffer, "{} - ", level_label(&log_level));
        let _ = framebuffer.write_fmt(args);
    }
}

/// Push a log entry into the queue. Formats into a fixed-size stack buffer
/// (no heap allocation) then pushes to the Vec (brief spinlock hold).
fn push_log_entry(sub_system: SubSystem, log_level: LogLevel, secs: u64, millis: u64, args: core::fmt::Arguments) {
    let mut entry = LogEntry {
        sub_system,
        log_level,
        length: 0,
        buffer: [0; 256],
    };
    {
        let mut writer = EntryWriter { entry: &mut entry };
        let _ = write!(writer, "{}.{:03}s {} - ", secs, millis, level_label(&log_level));
        let _ = writer.write_fmt(args);
    }
    LOG_QUEUE.lock().push(entry);
}

struct EntryWriter<'a> {
    entry: &'a mut LogEntry,
}

impl core::fmt::Write for EntryWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.entry.buffer.len() - self.entry.length;
        let to_copy = bytes.len().min(remaining);
        self.entry.buffer[self.entry.length..self.entry.length + to_copy]
            .copy_from_slice(&bytes[..to_copy]);
        self.entry.length += to_copy;
        Ok(())
    }
}

/// Log sink: drain all pending entries, write to framebuffer, yield.
/// Called from a dedicated kernel task (spawned by main).
pub fn log_sink_loop() -> ! {
    loop {
        // Drain all pending entries
        let entries = core::mem::take(&mut *LOG_QUEUE.lock());

        if !entries.is_empty() {
            if let Some(ref mut framebuffer) = *FRAMEBUFFER.lock() {
                for entry in &entries {
                    let text = unsafe { core::str::from_utf8_unchecked(&entry.buffer[..entry.length]) };
                    let (sr, sg, sb) = subsystem_rgb(&entry.sub_system);
                    framebuffer.set_color(sr, sg, sb);
                    let _ = write!(framebuffer, "[{}] ", entry.sub_system);

                    let (lr, lg, lb) = level_rgb(&entry.log_level);
                    framebuffer.set_color(lr, lg, lb);
                    let _ = framebuffer.write_str(text);
                }
            }
        }

        // Yield after draining — let other tasks run
        crate::scheduler::yield_current();
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
        write!(f, "{}", name)
    }
}

fn subsystem_ansi(ss: &SubSystem) -> &'static str {
    match ss {
        SubSystem::Kernel => "38;5;210",
        SubSystem::Boot => "38;5;150",
        SubSystem::X86_64 => "38;5;222",
        SubSystem::Physical => "38;5;111",
        SubSystem::KernelMemory => "38;5;183",
    }
}

fn subsystem_rgb(ss: &SubSystem) -> (u8, u8, u8) {
    match ss {
        SubSystem::Kernel => (240, 160, 160),
        SubSystem::Boot => (160, 220, 160),
        SubSystem::X86_64 => (230, 210, 140),
        SubSystem::Physical => (140, 180, 240),
        SubSystem::KernelMemory => (200, 170, 240),
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
        LogLevel::Debug => "38;5;243",
        LogLevel::Information => "38;5;252",
        LogLevel::Warning => "38;5;222",
        LogLevel::Error => "38;5;210",
    }
}

fn level_rgb(ll: &LogLevel) -> (u8, u8, u8) {
    match ll {
        LogLevel::Debug => (140, 140, 140),
        LogLevel::Information => (210, 210, 210),
        LogLevel::Warning => (240, 210, 130),
        LogLevel::Error => (240, 150, 150),
    }
}

// ---------------------------------------------------------------------------
// Core print function
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub fn _print(sub_system: SubSystem, log_level: LogLevel, args: ::core::fmt::Arguments) {
    let (secs, millis) = elapsed();

    // Disable interrupts for the entire log call to prevent preemption
    // while holding the SERIAL or LOG_QUEUE spinlocks. On single-CPU
    // systems, preemption during a spinlock hold = deadlock. The total
    // time here is ~100µs (UART write) which is acceptable vs the 5ms
    // preemption timeslice.
    let interrupts_were_enabled = x86_64::instructions::interrupts::are_enabled();
    x86_64::instructions::interrupts::disable();

    // --- serial (synchronous, always available) ---
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

    // --- framebuffer ---
    if log_level >= FB_LOG_LEVEL {
        if crate::scheduler::task_mutex::is_scheduler_active() {
            // After scheduler is running: push to async queue (no lock contention)
            push_log_entry(sub_system, log_level, secs, millis, args);
        } else {
            // During boot: write directly (no scheduler, no preemption risk)
            write_framebuffer_sync(sub_system, log_level, secs, millis, args);
        }
    }

    if interrupts_were_enabled {
        x86_64::instructions::interrupts::enable();
    }
}

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! serial_print {
    ($ss:path, $ll:path, $($arg:tt)*) => ($crate::log::_print($ss, $ll, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_println {
    ($ss:path, $ll:path) => ($crate::log::_print($ss, $ll, format_args!("\n")));
    ($ss:path, $ll:path, $fmt:expr) => ($crate::log::_print($ss, $ll, format_args!(concat!($fmt, "\n"))));
    ($ss:path, $ll:path, $fmt:expr, $($arg:tt)*) => ($crate::log::_print($ss, $ll, format_args!(concat!($fmt, "\n"), $($arg)*)));
}
