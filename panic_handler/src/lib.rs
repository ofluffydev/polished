//! # Custom Panic Handler for Polished Kernel
//!
//! This crate provides a custom panic handler for use in low-level, `no_std` Rust environments such as OS kernels or bootloaders.
//!
//! ## How Rust Uses This Panic Handler
//! In Rust, when a panic occurs (e.g., via `panic!()` or an assertion failure), the compiler looks for a function marked with the `#[panic_handler]` attribute. In `no_std` environments, you must provide your own panic handler, as the standard library's default is unavailable. This function is called with a reference to a [`core::panic::PanicInfo`] struct, which contains information about the panic location and message.
//!
//! This implementation:
//! - Uses serial logging (via the `serial_logging` crate) to output panic information to a serial port, which is essential for debugging in early boot or kernel code where no display is available.
//! - Prints the panic location (file, line, column) and message, if available.
//! - Halts the CPU after logging, preventing further execution.
//!
//! ## Usage
//! Link this crate in your kernel or bootloader. When a panic occurs, information will be sent over the serial port for developer diagnostics.

#![no_std]

extern crate alloc;

use alloc::string::ToString;
use polished_serial_logging::serial_write_str;

/// Custom panic handler for the kernel.
///
/// # How it works
/// - This function is called automatically by Rust when a panic occurs.
/// - It logs a generic error message and detailed panic info over the serial port.
/// - It then halts the CPU to prevent further execution, as continuing after a panic is unsafe in kernel code.
///
/// # Arguments
/// * `info` - Reference to [`core::panic::PanicInfo`] containing the panic message and location.
///
/// # Note
/// The `#[panic_handler]` attribute tells the Rust compiler to use this function as the panic handler in `no_std` builds.
#[allow(dead_code)]
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    // Log a generic error message to the serial port.
    polished_serial_logging::error("Kernel panic occurred!");
    // Print detailed panic information (location, message) to the serial port.
    print_panic_info_serial(info);
    // Enter an infinite loop, halting the CPU to prevent further execution.
    loop {
        // Halt the CPU: 'cli' disables interrupts, 'hlt' halts the processor.
        unsafe { core::arch::asm!("cli", "hlt") };
    }
}

/// Print detailed panic information to the serial port for debugging.
///
/// This function extracts the file, line, column, and message from the [`core::panic::PanicInfo`] struct
/// and writes them to the serial port using the `serial_logging` crate. This is crucial for debugging
/// in environments where no display is available.
///
/// # Arguments
/// * `info` - Reference to [`core::panic::PanicInfo`] containing the panic details.
///
/// # Output Format
/// The output is formatted for clarity, with clear section markers and warnings for release builds.
pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    // Print a header to indicate a panic has occurred.
    serial_write_str("=== PANIC ===\n");

    // Warn if this is a release build (debug info may be limited).
    #[cfg(not(debug_assertions))]
    serial_write_str("[WARNING] This is a release build, panic information may be limited.\n");

    // Print the location of the panic, if available.
    if let Some(location) = info.location() {
        serial_write_str("Location: ");
        serial_write_str(location.file());
        serial_write_str(":");
        serial_write_str(location.line().to_string().as_str());
        serial_write_str(":");
        serial_write_str(location.column().to_string().as_str());
        serial_write_str("\n");
    } else {
        serial_write_str("Location: <unknown>\n");
    }

    // Print the panic message, if available.
    if let Some(message) = info.message().as_str() {
        serial_write_str("Message: ");
        serial_write_str(message);
        serial_write_str("\n");
    } else {
        serial_write_str("Message: <none>\n");
    }

    // Print a footer to mark the end of the panic info.
    serial_write_str("=============\n");
    serial_write_str("\n\n");
}
