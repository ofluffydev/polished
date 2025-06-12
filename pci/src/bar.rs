// PCI BAR (Base Address Register) handling for PCI devices
// This module provides functions to probe and enumerate PCI BARs, which are used to map device memory or I/O regions.

use crate::error::PciError;
use alloc::vec::Vec;

/// Information about a single PCI BAR (Base Address Register).
///
/// A BAR describes a region of memory or I/O space used by a PCI device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarInfo {
    /// The BAR index (0-5). Each PCI device can have up to 6 BARs.
    pub index: u8,
    /// True if this BAR is for I/O space, false if for memory space.
    pub is_io: bool,
    /// True if this BAR is a 64-bit memory BAR (spans two BAR slots).
    pub is_64bit: bool,
    /// The base address of the region described by this BAR.
    pub address: u64,
    /// The size (in bytes) of the region described by this BAR.
    pub size: u64,
}

/// Probe a single BAR for a given PCI device and return its information.
///
/// # Arguments
/// * `bus` - PCI bus number (0-255)
/// * `device` - PCI device number (0-31)
/// * `function` - PCI function number (0-7)
/// * `bar_index` - Which BAR to probe (0-5)
///
/// # Returns
/// * `Ok(BarInfo)` if the BAR is present and valid
/// * `Err(PciError::DeviceNotFound)` if the BAR is unused
/// * `Err(PciError::InvalidOffset)` if the index is out of range
/// * `Err(PciError::IoFailure)` for I/O errors (future-proof)
///
/// # Safety
/// This function performs raw I/O port access to PCI configuration space, which is inherently unsafe.
pub unsafe fn probe_bar(
    bus: u8,
    device: u8,
    function: u8,
    bar_index: u8,
) -> Result<BarInfo, PciError> {
    // There are only 6 BARs per PCI device
    if bar_index >= 6 {
        return Err(PciError::InvalidOffset);
    }
    // Each BAR is 4 bytes, starting at offset 0x10 in the PCI config space
    let bar_offset = 0x10 + bar_index * 4;
    // Read the original BAR value
    let orig = unsafe { super::read_config_u32(bus, device, function, bar_offset) };
    // Write all 1s to the BAR to probe the size (per PCI spec)
    unsafe { super::write_config_u32(bus, device, function, bar_offset, 0xFFFF_FFFF) };
    // Read back the value to get the size mask
    let probed = unsafe { super::read_config_u32(bus, device, function, bar_offset) };
    // Restore the original BAR value
    unsafe { super::write_config_u32(bus, device, function, bar_offset, orig) };

    // If the BAR is zero, it's unused
    if orig == 0 {
        return Err(PciError::DeviceNotFound);
    }

    // The least significant bit determines if this is I/O (1) or memory (0)
    let is_io = (orig & 1) != 0;
    let mut is_64bit = false;
    let (address, size) = if is_io {
        // I/O BAR: bits 31-2 are the address, bits 1-0 are flags
        let address = (orig & 0xFFFFFFFC) as u64;
        // The size mask is also in bits 31-2
        let size = (!(probed & 0xFFFFFFFC)).wrapping_add(1) as u64;
        (address, size)
    } else {
        // Memory BAR: bits 3-1 are type, bits 0 is always 0
        let bar_type = (orig >> 1) & 0b11;
        if bar_type == 0b10 {
            // 64-bit memory BAR: occupies two BAR slots
            is_64bit = true;
            // Read the high 32 bits of the BAR
            let orig_hi = unsafe { super::read_config_u32(bus, device, function, bar_offset + 4) };
            // Probe the high 32 bits for size
            unsafe { super::write_config_u32(bus, device, function, bar_offset + 4, 0xFFFF_FFFF) };
            let probed_hi =
                unsafe { super::read_config_u32(bus, device, function, bar_offset + 4) };
            // Restore the original high 32 bits
            unsafe { super::write_config_u32(bus, device, function, bar_offset + 4, orig_hi) };
            // Combine low and high parts for full 64-bit address
            let address = ((orig_hi as u64) << 32) | ((orig & 0xFFFF_FFF0) as u64);
            // Combine size mask from both low and high parts
            let size_lo = probed & 0xFFFF_FFF0;
            let size_hi = probed_hi as u64;
            let size_full = (!(size_hi << 32 | size_lo as u64)).wrapping_add(1) as u64;
            (address, size_full)
        } else {
            // 32-bit memory BAR
            let address = (orig & 0xFFFF_FFF0) as u64;
            let size = (!(probed & 0xFFFF_FFF0)).wrapping_add(1) as u64;
            (address, size)
        }
    };
    // If the size is zero, the BAR is not valid
    if size == 0 {
        return Err(PciError::DeviceNotFound);
    }
    Ok(BarInfo {
        index: bar_index,
        is_io,
        is_64bit,
        address,
        size,
    })
}

/// Get all valid BARs for a PCI device (up to 6).
///
/// # Arguments
/// * `bus` - PCI bus number
/// * `device` - PCI device number
/// * `function` - PCI function number
///
/// # Returns
/// * Ok(Vec<BarInfo>) for all valid BARs
/// * Err(PciError) if any error occurs (first error encountered)
///
/// # Safety
/// This function performs raw I/O port access and is unsafe.
pub unsafe fn get_bars(bus: u8, device: u8, function: u8) -> Result<Vec<BarInfo>, PciError> {
    let mut bars = Vec::new();
    let mut i = 0;
    while i < 6 {
        match unsafe { probe_bar(bus, device, function, i) } {
            Ok(bar) => {
                bars.push(bar);
                // 64-bit BARs occupy two slots, so skip the next index
                if bar.is_64bit {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            Err(PciError::DeviceNotFound) => {
                i += 1;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(bars)
}
