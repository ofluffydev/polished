#![no_std]

extern crate alloc;

use alloc::string::ToString;
use serial_logging::serial_write_str;

#[allow(dead_code)]
#[cfg_attr(not(test), panic_handler)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_logging::error("Kernel panic occurred!");
    print_panic_info_serial(info);
    loop {
        // Halt the CPU to prevent further execution
        unsafe { core::arch::asm!("cli", "hlt") };
    }
}

pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    serial_write_str("=== PANIC ===\n");

    #[cfg(not(debug_assertions))]
    serial_write_str("[WARNING] This is a release build, panic information may be limited.\n");

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

    if let Some(message) = info.message().as_str() {
        serial_write_str("Message: ");
        serial_write_str(message);
        serial_write_str("\n");
    } else {
        serial_write_str("Message: <none>\n");
    }

    serial_write_str("=============\n");
    serial_write_str("\n\n");
}
