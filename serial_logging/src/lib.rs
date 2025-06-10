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

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Logs a message to the serial port with a given log level prefix.
///
/// # Examples
/// ```
/// serial_log!("[INFO] ", "Hello, world!");
/// serial_log!("[DEBUG] ", "Value: {}", 42);
/// ```
#[macro_export]
macro_rules! serial_log {
    ($level:expr, $msg:expr) => {
        $crate::serial_write_str($level);
        $crate::serial_write_str($msg);
        $crate::serial_write_str("\r\n");
    };
    ($level:expr, $fmt:expr, $($arg:tt)*) => {{
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
pub fn enable_serial_logging() {
    unsafe {
        LOGGING_ENABLED = true;
    }
}

/// Disables serial logging output.
pub fn disable_serial_logging() {
    unsafe {
        LOGGING_ENABLED = false;
    }
}

/// Returns whether serial logging is currently enabled.
pub fn is_serial_logging_enabled() -> bool {
    unsafe { LOGGING_ENABLED }
}

/// Writes a single byte to the serial port.
///
/// Blocks until the port is ready to accept a byte.
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

/// Writes a string to the serial port.
///
/// Each byte of the string is sent individually.
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
/// Does not include a `0x` prefix.
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
/// # Examples
/// ```
/// serial::info("System started");
/// ```
pub fn info(text: &str) {
    serial_log!("[INFO] ", "{}", text);
}

/// Logs an info-level hexadecimal value to the serial port.
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
/// # Examples
/// ```
/// serial::error("An error occurred");
/// ```
pub fn error(text: &str) {
    serial_log!("[ERROR] ", "{}", text);
}

/// Logs a warning-level message to the serial port.
///
/// # Examples
/// ```
/// serial::warning("Low disk space");
/// ```
pub fn warn(text: &str) {
    serial_log!("[WARNING] ", "{}", text);
}

/// Logs an error-level message to the serial port with formatting support.
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
