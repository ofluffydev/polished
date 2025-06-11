//! # Serial Logging Library for x86_64 Kernels
//!
//! This crate provides robust serial port logging for kernel and bootloader environments, with a focus on QEMU usage. It offers macros and functions for formatted and raw serial output, as well as a minimal `kprint!` macro for early boot or no_std contexts where dependencies are limited.
//!
//! ## QEMU Serial Logging
//!
//! QEMU can redirect the guest's serial port (COM1, 0x3F8) to the host's standard output or a file. This allows you to see kernel logs by running QEMU with `-serial stdio` or `-serial file:output.log`. All output sent to the serial port will appear in your terminal or the specified file, making debugging much easier, especially before graphics or higher-level logging is available.
//!
//! ## When to Use `serial_log!`/`serial_print!` vs `kprint!`
//!
//! - Use `serial_log!`, `serial_print!`, and related macros for most kernel logging. These use a full-featured serial driver (with buffering, formatting, and thread safety) and are suitable once the kernel is initialized.
//! - Use `kprint!` for minimal, dependency-free output in very early boot stages, or in environments where only core formatting is available. `kprint!` writes directly to the serial port using inline assembly, bypassing all higher-level abstractions.
//!
//! ## Features
//! - Thread-safe, formatted serial output via `serial_print!`, `serial_log!`, etc.
//! - Log level macros for info, warning, and error messages.
//! - Hexadecimal logging support.
//! - Minimal `kprint!` macro for direct serial output.
//! - Enable/disable logging at runtime.
//!
//! ## Example (QEMU):
//!
//! ```sh
//! qemu-system-x86_64 -serial stdio -kernel path/to/kernel
//! ```
//!
//! In your kernel code:
//!
//! ```rust
//! serial_println!("Hello, QEMU serial!");
//! serial_log!("[INFO] ", "Boot complete");
//! ```

#![no_std]

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

pub mod kprint;

pub use crate::kprint::DebugSerial;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface (COM1, 0x3F8).
///
/// This macro uses the main serial driver, which is thread-safe and supports formatting.
/// Output will appear in QEMU's terminal if run with `-serial stdio`.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
///
/// Equivalent to `serial_print!` but adds a `\n` at the end.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Logs a message to the serial port with a given log level prefix.
///
/// - `$level`: Prefix string (e.g., "[INFO] ").
/// - `$msg`: Message string or format string.
/// - Additional arguments are formatted as in `format!`.
///
/// # Examples
/// ```
/// serial_log!("[INFO] ", "Hello, world!");
/// serial_log!("[DEBUG] ", "Value: {}", 42);
/// ```
#[macro_export]
macro_rules! serial_log {
    ($level:expr, $msg:expr) => {
        // Writes a log message with a prefix and message.
        $crate::serial_write_str($level);
        $crate::serial_write_str($msg);
        $crate::serial_write_str("\r\n");
    };
    ($level:expr, $fmt:expr, $($arg:tt)*) => {{
        // Writes a formatted log message with a prefix.
        use core::fmt::Write;
        struct SerialLogger;
        impl core::fmt::Write for SerialLogger {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                $crate::serial_write_str(s);
                Ok(())
            }
        }
        $crate::serial_write_str($level);
        let _ = write!(SerialLogger, $fmt, $($arg)*);
        $crate::serial_write_str("\r\n");
    }};
}

/// Logs a hexadecimal value to the serial port with a given log level prefix.
///
/// - `$level`: Prefix string (e.g., "[INFO] ").
/// - `$value`: Value to print in hexadecimal.
///
/// # Examples
/// ```
/// serial_log_hex!("[INFO] ", 0xdeadbeef);
/// ```
#[macro_export]
macro_rules! serial_log_hex {
    ($level:expr, $value:expr) => {
        $crate::serial_write_str($level);
        $crate::serial_write_str("0x");
        $crate::serial_write_hex($value);
        $crate::serial_write_str("\r\n");
    };
}

const SERIAL_PORT: u16 = 0x3F8; // COM1

static mut LOGGING_ENABLED: bool = true;

/// Enables serial logging output.
///
/// When disabled, all serial output functions become no-ops.
pub fn enable_serial_logging() {
    unsafe {
        LOGGING_ENABLED = true;
    }
}

/// Disables serial logging output.
///
/// When disabled, all serial output functions become no-ops.
pub fn disable_serial_logging() {
    unsafe {
        LOGGING_ENABLED = false;
    }
}

/// Returns whether serial logging is currently enabled.
///
/// This can be used to temporarily silence serial output.
pub fn is_serial_logging_enabled() -> bool {
    unsafe { LOGGING_ENABLED }
}

/// Writes a single byte to the serial port (COM1, 0x3F8).
///
/// Blocks until the port is ready to accept a byte. Used internally by all higher-level output functions.
///
/// # QEMU
/// Output will appear in the QEMU terminal if run with `-serial stdio`.
pub fn serial_write_byte(byte: u8) {
    if !is_serial_logging_enabled() {
        return;
    }
    unsafe {
        let mut line_status = Port::<u8>::new(SERIAL_PORT + 5);
        while (line_status.read() & 0x20) == 0 {}
        let mut data = Port::new(SERIAL_PORT);
        data.write(byte);
    }
}

/// Writes a string to the serial port, byte by byte.
///
/// Used by all higher-level output macros and functions.
pub fn serial_write_str(s: &str) {
    if !is_serial_logging_enabled() {
        return;
    }
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

/// Writes a hexadecimal representation of a `u64` value to the serial port.
///
/// Does not include a `0x` prefix. Used by `serial_log_hex!` and similar macros.
pub fn serial_write_hex(mut value: u64) {
    if !is_serial_logging_enabled() {
        return;
    }
    let mut buf = [0u8; 16];
    let mut i = buf.len();
    if value == 0 {
        serial_write_str("0");
        return;
    }
    while value != 0 {
        i -= 1;
        let digit = (value & 0xF) as u8;
        buf[i] = match digit {
            0..=9 => b'0' + digit,
            10..=15 => b'A' + (digit - 10),
            _ => b'?', // Should not happen
        };
        value >>= 4;
    }
    serial_write_str(core::str::from_utf8(&buf[i..]).unwrap());
}

/// Logs an info-level message to the serial port.
///
/// Equivalent to `serial_log!("[INFO] ", ...)`.
///
/// # Examples
/// ```
/// serial::info("System started");
/// ```
pub fn info(text: &str) {
    serial_log!("[INFO] ", "{}", text);
}

/// Logs an info-level hexadecimal value to the serial port.
///
/// Equivalent to `serial_log_hex!("[INFO] ", value)`.
///
/// # Examples
/// ```
/// serial::info_hex(0xdeadbeef);
/// ```
pub fn info_hex(value: u64) {
    serial_log_hex!("[INFO] ", value);
}

/// Logs an error-level message to the serial port.
///
/// Equivalent to `serial_log!("[ERROR] ", ...)`.
///
/// # Examples
/// ```
/// serial::error("An error occurred");
/// ```
pub fn error(text: &str) {
    serial_log!("[ERROR] ", "{}", text);
}

/// Logs a warning-level message to the serial port.
///
/// Equivalent to `serial_log!("[WARNING] ", ...)`.
///
/// # Examples
/// ```
/// serial::warn("Low disk space");
/// ```
pub fn warn(text: &str) {
    serial_log!("[WARNING] ", "{}", text);
}

/// Logs an error-level message to the serial port with formatting support.
///
/// Equivalent to `serial_log!("[ERROR] ", ...)` but as a macro for formatting.
///
/// # Examples
/// ```
/// error!("Failed: {}", 42);
/// ```
#[macro_export]
macro_rules! error {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_log!("[ERROR] ", $fmt, $($arg)*);
    };
    ($msg:expr) => {
        $crate::serial_log!("[ERROR] ", $msg);
    };
}
