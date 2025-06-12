// This module provides functions for scanning and enumerating PCI devices on bus 0.
// PCI (Peripheral Component Interconnect) is a standard for connecting peripherals to a computer's motherboard.
// In OS development, enumerating PCI devices is a key step in discovering hardware present in the system.

use crate::{
    device::PciDevice,
    error::PciError,
    logger::{DefaultLogger, Logger},
    lookup::{class_code_str, subclass_str, vendor_id_str},
};
use alloc::format;
use alloc::vec::Vec;

/// Scan all devices on PCI bus 0 and return a vector of PciDevice structs.
///
/// This function iterates over all possible device numbers (0..31) on PCI bus 0, checks if a device is present,
/// and if so, reads its configuration space to gather information such as vendor ID, device ID, class code, etc.
///
/// # Returns
/// Ok(Vec<PciDevice>) if devices are found, or Err(PciError) if none or on error.
pub fn scan_bus0_devices() -> Result<Vec<PciDevice>, PciError> {
    let mut devices = Vec::new();
    let mut found = false;
    // PCI allows up to 32 devices per bus (device numbers 0..31)
    for device in 0u8..32 {
        // Read the vendor ID from the PCI configuration space.
        // If the vendor ID is 0xFFFF, there is no device present at this slot.
        let vendor_id = unsafe { crate::pci_config_read(0, device, 0, 0) & 0xFFFF } as u16;
        if vendor_id == 0xFFFF {
            continue;
        }
        found = true;
        // Read other fields from the configuration space.
        let device_id = ((unsafe { crate::pci_config_read(0, device, 0, 0) }) >> 16) as u16;
        let class = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 24) as u8;
        let subclass = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 16) as u8;
        let prog_if = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 8) as u8;
        let header_type = ((unsafe { crate::pci_config_read(0, device, 0, 0xC) }) >> 16) as u8;
        // Store the device information in a struct for later use.
        devices.push(PciDevice {
            bus: 0,
            device,
            function: 0,
            vendor_id,
            device_id,
            class,
            subclass,
            prog_if,
            header_type,
        });
    }
    if found {
        Ok(devices)
    } else {
        Err(PciError::DeviceNotFound)
    }
}

/// Enumerate all PCI devices on bus 0 and print their information.
///
/// This function is a demonstration of how to scan for PCI devices and print their details.
/// It uses the logger to output information, which is helpful for debugging in early OS development
/// when you may not have a full user interface.
pub fn pci_enumeration_demo() {
    // Log the start of the enumeration process.
    DefaultLogger::info("PCI enumeration: Scanning bus 0...");
    // Iterate over all possible device numbers on bus 0.
    for device in 0u8..32 {
        // Read the vendor ID to check if a device is present.
        let vendor_id = unsafe { crate::pci_config_read(0, device, 0, 0) & 0xFFFF } as u16;
        if vendor_id == 0xFFFF {
            continue;
        }
        // Read device ID, class, and subclass for display.
        let device_id = ((unsafe { crate::pci_config_read(0, device, 0, 0) }) >> 16) as u16;
        let class = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 24) as u8;
        let subclass = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 16) as u8;
        // Print the device information using a helper function.
        crate::print_pci_device(0, device, vendor_id, device_id, class, subclass);
    }
}

/// Print information about a PCI device.
///
/// This function logs details about a single PCI device, such as its bus/device number, vendor, device ID,
/// class code, and subclass. This is useful for debugging and for learning about the hardware present in the system.
///
/// # Arguments
/// * `bus` - The PCI bus number (usually 0 for simple systems)
/// * `device` - The device number on the bus (0..31)
/// * `vendor_id` - The vendor ID read from the device
/// * `device_id` - The device ID read from the device
/// * `class_code` - The class code (identifies the type of device, e.g., network, storage, etc.)
/// * `subclass` - The subclass code (further refines the device type)
pub fn print_pci_device(
    bus: u8,
    device: u8,
    vendor_id: u16,
    device_id: u16,
    class_code: u8,
    subclass: u8,
) {
    // Log each piece of information for the device.
    DefaultLogger::info("PCI device found:");
    DefaultLogger::info(&crate::format_bus(bus)); // e.g., "  bus=0"
    DefaultLogger::info(&crate::format_device(device)); // e.g., "  device=5"
    DefaultLogger::info(vendor_id_str(vendor_id)); // e.g., "  vendor=Intel"
    DefaultLogger::info(&format!("  device_id=0x{device_id:04x}")); // e.g., "  device_id=0x1234"
    DefaultLogger::info(class_code_str(class_code)); // e.g., "  class=Network controller"
    DefaultLogger::info(subclass_str(subclass)); // e.g., "  subclass=Ethernet controller"
}
