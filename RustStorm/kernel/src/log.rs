use core::ops::Sub;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

const SHOW_LOG_LEVEL: LogLevel = LogLevel::Debug;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(sub_system: SubSystem, log_level: LogLevel, args: ::core::fmt::Arguments) {
    fn get_sub_system_color(sub_system: &SubSystem) -> &'static str {
        match sub_system {
            SubSystem::Kernel => "1;31",
            SubSystem::Boot => "1;32",
            SubSystem::X86_64 => "1;33",
            SubSystem::Physical => "1;34",
            SubSystem::KernelMemory => "1;35",
        }
    }

    fn get_log_level_color(log_level: &LogLevel) -> &'static str {
        match log_level {
            LogLevel::Debug => "38;5;8",
            LogLevel::Information => "0",
            LogLevel::Warning => "33",
            LogLevel::Error => "31",
        }
    }

    use core::fmt::Write;
    if log_level as usize >= SHOW_LOG_LEVEL as usize { // check 
        let mut serial = SERIAL1.lock();
        serial.write_fmt(format_args!("\x1b[0m[\x1b[{}m{:?}\x1b[0m]/{:?}: \x1b[{}m", get_sub_system_color(&sub_system), sub_system, log_level, get_log_level_color(&log_level))).expect("Printing to serial failed");
        serial.write_fmt(args).expect("Printing to serial failed");
    }
}

#[derive(Debug)]
pub enum SubSystem {
    Kernel,
    Boot,
    X86_64,
    Physical,
    KernelMemory,
}

#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Debug,
    Information,
    Warning,
    Error,
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($ss:path, $ll:path, $($arg:tt)*) => ($crate::log::_print($ss, $ll, format_args!($($arg)*)));
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! log_println {
    ($ss:path, $ll:path) => ($crate::serial_print!($ss, $ll, "\n"));
    ($ss:path, $ll:path, $fmt:expr) => ($crate::serial_print!($ss, $ll, concat!($fmt, "\n")));
    ($ss:path, $ll:path, $fmt:expr, $($arg:tt)*) => ($crate::serial_print!($ss, $ll, concat!($fmt, "\n"), $($arg)*));
}
