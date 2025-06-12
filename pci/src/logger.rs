// Logger trait and implementations for PCI logging abstraction
// This module provides a flexible logging interface for the PCI crate, allowing different logging backends
// to be selected at compile time using Cargo features. This is useful in OS development where you may want
// to log to serial, to a host log crate, or disable logging entirely depending on your build environment.

// --- SerialLogger: Uses the polished_serial_logging crate for output ---
#[cfg(feature = "polished_serial_logging")]
/// Internal: Not part of the public API. Use `DefaultLogger` instead.
pub struct SerialLogger;

#[cfg(feature = "polished_serial_logging")]
impl Logger for SerialLogger {
    /// Logs an info-level message to the serial port.
    ///
    /// # Arguments
    /// * `msg` - The message to log.
    fn info(msg: &str) {
        // Calls the info function from the serial logging crate.
        polished_serial_logging::info(msg);
    }
}

// --- LogCrateLogger: Uses the standard Rust `log` crate ---
#[cfg(feature = "logger_log")]
/// Internal: Not part of the public API. Use `DefaultLogger` instead.
pub struct LogCrateLogger;

#[cfg(feature = "logger_log")]
impl Logger for LogCrateLogger {
    /// Logs an info-level message using the `log` crate.
    ///
    /// # Arguments
    /// * `msg` - The message to log.
    fn info(msg: &str) {
        // The log crate uses macros for logging. The curly braces are used for formatting.
        log::info!("{}", msg);
    }
}

// --- NoopLogger: Disables logging (does nothing) ---
#[cfg(all(not(feature = "polished_serial_logging"), not(feature = "logger_log")))]
/// Internal: Not part of the public API. Use `DefaultLogger` instead.
pub struct NoopLogger;

#[cfg(all(not(feature = "polished_serial_logging"), not(feature = "logger_log")))]
impl Logger for NoopLogger {
    /// Ignores the log message. No output is produced.
    fn info(_msg: &str) {}
}

/// The `Logger` trait defines the interface for logging in the PCI crate.
///
/// By using a trait, we can swap out different logging implementations at compile time
/// depending on which Cargo features are enabled. This is a common pattern in OS development
/// to keep code portable and flexible.
pub trait Logger {
    /// Logs an info-level message.
    ///
    /// # Arguments
    /// * `msg` - The message to log.
    fn info(msg: &str);
}

// --- DefaultLogger type alias ---
// This type alias selects the appropriate logger implementation based on enabled features.
// You can use `DefaultLogger` in your code to refer to the active logger without worrying about which backend is used.

#[cfg(feature = "polished_serial_logging")]
pub type DefaultLogger = SerialLogger;

#[cfg(all(not(feature = "polished_serial_logging"), feature = "logger_log"))]
pub type DefaultLogger = LogCrateLogger;

#[cfg(all(not(feature = "polished_serial_logging"), not(feature = "logger_log")))]
pub type DefaultLogger = NoopLogger;
