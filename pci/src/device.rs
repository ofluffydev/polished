// This module defines the PciDevice struct and related functionality for representing and summarizing PCI devices.
// It is intended for use in OS development, where direct hardware access and device enumeration are common tasks.
//
// PCI (Peripheral Component Interconnect) is a standard for connecting peripherals to a computer's processor and memory.
// Each PCI device is identified by a bus, device, and function number, and has associated vendor and device IDs.

use core::fmt::Debug;

use alloc::string::String; // Use heap-allocated strings from the alloc crate (common in no_std environments)

/// Represents a single PCI device on the system.
///
/// The fields correspond to the standard PCI configuration space header fields.
/// - `bus`, `device`, `function`: Identify the device's location on the PCI bus hierarchy.
/// - `vendor_id`, `device_id`: Uniquely identify the hardware vendor and device type.
/// - `class`, `subclass`, `prog_if`: Describe the device's function (e.g., mass storage, network, etc.).
/// - `header_type`: Indicates the layout of the device's configuration space.
#[derive(Clone, Copy)]
pub struct PciDevice {
    /// PCI bus number (0-255)
    pub bus: u8,
    /// PCI device number on the bus (0-31)
    pub device: u8,
    /// PCI function number (0-7, for multifunction devices)
    pub function: u8,
    /// Vendor ID (assigned by PCI-SIG)
    pub vendor_id: u16,
    /// Device ID (assigned by the vendor)
    pub device_id: u16,
    /// Class code (broad category, e.g., 0x01 = mass storage)
    pub class: u8,
    /// Subclass code (specific type within the class)
    pub subclass: u8,
    /// Programming interface (further describes device capabilities)
    pub prog_if: u8,
    /// Header type (0 = standard, 1 = PCI-to-PCI bridge, etc.)
    pub header_type: u8,
}

// Implement the Debug trait for PciDevice so it can be printed with {:?} in debug output.
// This is useful for logging and debugging during OS development.
impl Debug for PciDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            // Print all fields in a readable format, with hex formatting for IDs and codes.
            "PciDevice {{ bus: {}, device: {}, function: {}, vendor_id: {:#06x}, device_id: {:#06x}, class: {:#04x}, subclass: {:#04x}, prog_if: {:#04x}, header_type: {:#04x} }}",
            self.bus,
            self.device,
            self.function,
            self.vendor_id,
            self.device_id,
            self.class,
            self.subclass,
            self.prog_if,
            self.header_type
        )
    }
}

impl PciDevice {
    /// Returns a short summary string for the device, suitable for logging or display.
    ///
    /// Example output:
    ///   PCI 00:1f.2 vendor=8086 device=2922 class=01 subclass=06
    ///
    /// This can help you quickly identify devices during PCI enumeration.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        use core::fmt::Write; // Import the Write trait for formatting into a String
        // Write formatted device info into the string. The _ = is used to ignore the Result,
        // since formatting into a String should not fail in this context.
        let _ = write!(
            s,
            "PCI {:02x}:{:02x}.{} vendor={:04x} device={:04x} class={:02x} subclass={:02x}",
            self.bus,
            self.device,
            self.function,
            self.vendor_id,
            self.device_id,
            self.class,
            self.subclass
        );
        s
    }
}
