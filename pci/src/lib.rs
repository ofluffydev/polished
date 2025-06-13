//! PCI Bus Access and Enumeration Library
//!
//! This crate provides basic PCI bus access and enumeration routines for x86 systems.
//! It is designed for use in `no_std` environments, such as kernels or bootloaders.
//!
//! # Features
//! - Read PCI configuration space
//! - Enumerate devices on PCI bus 0
//! - Print device information using a serial logger
//!
//! # Example
//! ```no_run
//! use polished_pci::pci_enumeration_demo;
//! pci_enumeration_demo();
//! ```
//!
//! # Integration Examples
//!
//! ## Kernel-level Initialization
//!
//! ```no_run
//! // In your kernel's main.rs or init.rs
//! use polished_pci::pci_enumeration_demo;
//! // Initialize and enumerate PCI devices (prints to serial logger)
//! pci_enumeration_demo();
//! ```
//!
//! ## Dumping All PCI Devices
//!
//! ```no_run
//! use polished_pci::scan_bus0_devices;
//!
//! // Scan all devices on bus 0
//! let devices = scan_bus0_devices();
//! for dev in devices.iter() {
//!     // Use the fields of dev directly, or print as needed
//!     // e.g., println!("bus={} device={} vendor={:04x} device={:04x} class={:02x} subclass={:02x}", dev.bus, dev.device, dev.vendor_id, dev.device_id, dev.class, dev.subclass);
//! }
//! ```
//!
//! ## Finding the First Network Device
//!
//! ```no_run
//! use polished_pci::{scan_bus0_devices, PciDevice, class_code_str};
//!
//! // Scan all devices on bus 0
//! let devices = scan_bus0_devices().unwrap();
//! // Class code 0x02 is for network controllers
//! let net_dev = devices.iter().find(|dev| dev.class == 0x02);
//! if let Some(dev) = net_dev {
//!     // Do something with the network device
//!     // e.g., print info or initialize driver
//! }
//! ```

#![no_std] // Do not link the Rust standard library. Required for OS/embedded development.

extern crate alloc; // Import the alloc crate for heap-allocated types (e.g., String, Vec)

// --- Module Declarations ---
// Each module below provides a logical grouping of PCI-related functionality.
// These are split into separate files for clarity and maintainability.

pub mod bar; // Base Address Register (BAR) handling
pub mod config; // PCI configuration space access (read/write)
pub mod device; // PCI device struct and helpers
pub mod error; // Error types for PCI operations
pub mod lookup; // Lookup tables for vendor/class names
pub mod scan; // PCI bus scanning and enumeration routines

mod logger; // Logging interface (e.g., serial output)

// --- Re-exports ---
// These 'pub use' statements make selected types and functions available at the crate root.
// This allows users to write `use polished_pci::PciDevice;` instead of `use polished_pci::device::PciDevice;`

pub use bar::{BarInfo, get_bars, probe_bar}; // now return Result types
pub use config::{
    pci_config_read, read_config_u8, read_config_u16, read_config_u32, write_config_u8,
    write_config_u16, write_config_u32,
};
pub use device::PciDevice;
pub use error::PciError;
pub use lookup::{class_code_str, subclass_str, vendor_id_str};
pub use scan::{pci_enumeration_demo, print_pci_device, scan_bus0_devices};

// --- Utility Functions ---
// These are simple helpers for formatting PCI bus/device numbers as strings.
// They are not part of the public API, but are used internally for display/logging.

use alloc::string::String;

/// Format a PCI bus number as a string for display/logging.
/// Example: format_bus(2) -> "  bus=2"
fn format_bus(bus: u8) -> String {
    use alloc::string::ToString;
    let mut s = "  bus=".to_string();
    s.push_str(&bus.to_string());
    s
}

/// Format a PCI device number as a string for display/logging.
/// Example: format_device(5) -> "  device=5"
fn format_device(device: u8) -> String {
    use alloc::string::ToString;
    let mut s = "  device=".to_string();
    s.push_str(&device.to_string());
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_device_summary() {
        let dev = PciDevice {
            bus: 0,
            device: 1,
            function: 0,
            vendor_id: 0x8086,
            device_id: 0x1234,
            class: 0x02,
            subclass: 0x00,
            prog_if: 0x00,
            header_type: 0x00,
        };
        let summary = dev.summary();
        assert!(summary.contains("PCI 00:01.0"));
        assert!(summary.contains("vendor=8086"));
        assert!(summary.contains("device=1234"));
        assert!(summary.contains("class=02"));
        assert!(summary.contains("subclass=00"));
    }

    #[test]
    fn test_vendor_id_str() {
        assert!(vendor_id_str(0x8086).contains("Intel"));
        assert!(vendor_id_str(0x10DE).contains("NVIDIA"));
        assert_eq!(vendor_id_str(0xFFFF), "  vendor=unknown");
    }

    #[test]
    fn test_class_code_str() {
        assert!(class_code_str(0x01).contains("Mass Storage"));
        assert!(class_code_str(0x02).contains("Network"));
        assert_eq!(class_code_str(0xFF), "  class=other");
    }

    #[test]
    fn test_subclass_str() {
        assert!(subclass_str(0x80).contains("Other"));
        assert_eq!(subclass_str(0x05), "  subclass=0x05");
        assert_eq!(subclass_str(0xFF), "  subclass=other");
    }

    #[test]
    fn test_pci_error_eq() {
        assert_eq!(PciError::DeviceNotFound, PciError::DeviceNotFound);
        assert_ne!(PciError::DeviceNotFound, PciError::IoFailure);
    }

    #[test]
    fn test_format_bus_and_device() {
        assert_eq!(format_bus(2), "  bus=2");
        assert_eq!(format_device(5), "  device=5");
    }
}
