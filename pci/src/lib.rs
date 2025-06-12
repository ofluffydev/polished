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

#![no_std]

extern crate alloc;

#[cfg(feature = "polished_serial_logging")]
use polished_serial_logging::info;

#[cfg(not(feature = "polished_serial_logging"))]
use core::arch::asm;
// If the polished_serial_logging feature is not enabled, define a no-op info function
#[cfg(not(feature = "polished_serial_logging"))]
fn info(_msg: &str) {
    unsafe {
        asm!(
            "nop", // No-op to avoid unused function warning
        );
    }
}

/// I/O port for PCI configuration address
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
/// I/O port for PCI configuration data
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Read a 32-bit value from PCI configuration space.
///
/// # Safety
/// This function performs raw I/O port access and is unsafe.
///
/// # Arguments
/// * `bus` - PCI bus number (0-255)
/// * `device` - PCI device number (0-31)
/// * `function` - PCI function number (0-7)
/// * `offset` - Register offset (must be 4-byte aligned)
unsafe fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    // Write address to PCI_CONFIG_ADDRESS port
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
    }
    // Read data from PCI_CONFIG_DATA port
    unsafe { inl(PCI_CONFIG_DATA) }
}

/// Write a 32-bit value to an I/O port.
///
/// # Safety
/// This function uses inline assembly to access hardware ports.
unsafe fn outl(port: u16, val: u32) {
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") val);
    }
}

/// Read a 32-bit value from an I/O port.
///
/// # Safety
/// This function uses inline assembly to access hardware ports.
unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    unsafe {
        core::arch::asm!("in eax, dx", in("dx") port, out("eax") val);
    }
    val
}

/// Enumerate all PCI devices on bus 0 and print their information.
///
/// This function scans all 32 possible devices on PCI bus 0, reads their vendor and device IDs,
/// and prints information about each present device using the serial logger.
///
/// # Example
/// ```no_run
/// use polished_pci::pci_enumeration_demo;
/// pci_enumeration_demo();
/// ```
pub fn pci_enumeration_demo() {
    info("PCI enumeration: Scanning bus 0...");
    for device in 0u8..32 {
        // Read vendor ID (0xFFFF means no device present)
        let vendor_id = unsafe { pci_config_read(0, device, 0, 0) & 0xFFFF } as u16;
        if vendor_id == 0xFFFF {
            continue; // No device present
        }
        // Read device ID, class code, and subclass
        let device_id = ((unsafe { pci_config_read(0, device, 0, 0) }) >> 16) as u16;
        let class_code = ((unsafe { pci_config_read(0, device, 0, 8) }) >> 24) as u8;
        let subclass = ((unsafe { pci_config_read(0, device, 0, 8) }) >> 16) as u8;
        print_pci_device(0, device, vendor_id, device_id, class_code, subclass);
    }
}

/// Print information about a PCI device.
///
/// This function prints each field of the PCI device as a separate line using the serial logger.
/// It provides human-readable names for common vendor and class codes.
fn print_pci_device(
    bus: u8,
    device: u8,
    vendor_id: u16,
    device_id: u16,
    class_code: u8,
    subclass: u8,
) {
    info("PCI device found:");
    info(&format_bus(bus));
    info(&format_device(device));
    info(match vendor_id {
        0x8086 => "  vendor=0x8086 (Intel)",
        0x10DE => "  vendor=0x10DE (NVIDIA)",
        0x1234 => "  vendor=0x1234 (QEMU)",
        0x1AF4 => "  vendor=0x1AF4 (Red Hat / QEMU (VirtIO))",
        _ => "  vendor=unknown",
    });
    {
        // Print device_id as a hex string
        use core::fmt::Write;
        struct Buffer {
            buf: [u8; 32],
            pos: usize,
        }
        impl core::fmt::Write for Buffer {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let len = bytes.len().min(self.buf.len() - self.pos);
                self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
                self.pos += len;
                Ok(())
            }
        }
        let mut buffer = Buffer {
            buf: [0u8; 32],
            pos: 0,
        };
        let _ = write!(&mut buffer, "  device_id=0x{device_id:04X}");
        let s = core::str::from_utf8(&buffer.buf[..buffer.pos]).unwrap_or("  device_id=?");
        info(s);
    }
    info(match class_code {
        0x01 => "  class=0x01 (Mass Storage Controller)",
        0x02 => "  class=0x02 (Network Controller)",
        0x03 => "  class=0x03 (Display Controller)",
        0x06 => "  class=0x06 (Bridge Device)",
        _ => "  class=other",
    });
    info(match subclass {
        0x00 => "  subclass=0x00",
        0x01 => "  subclass=0x01",
        0x02 => "  subclass=0x02",
        0x03 => "  subclass=0x03",
        0x04 => "  subclass=0x04",
        0x05 => "  subclass=0x05",
        0x06 => "  subclass=0x06",
        0x80 => "  subclass=0x80 (Other)",
        _ => "  subclass=other",
    });
}

/// Format the bus number for display.
///
/// # Arguments
/// * `bus` - PCI bus number
///
/// # Returns
/// A string like "  bus=0".
fn format_bus(bus: u8) -> alloc::string::String {
    use alloc::string::ToString;
    let mut s = "  bus=".to_string();
    s.push_str(&bus.to_string());
    s
}

/// Format the device number for display.
///
/// # Arguments
/// * `device` - PCI device number
///
/// # Returns
/// A string like "  device=5".
fn format_device(device: u8) -> alloc::string::String {
    use alloc::string::ToString;
    let mut s = "  device=".to_string();
    s.push_str(&device.to_string());
    s
}
