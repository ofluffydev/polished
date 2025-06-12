// PCI configuration space access routines for x86 systems
// This module provides safe wrappers and low-level routines for reading and writing PCI config space.
// It is designed for use in OS kernels and low-level system software.

use core::arch::asm;

// I/O port addresses for PCI configuration mechanism #1 (standard on x86 PCs)
const PCI_CONFIG_ADDRESS: u16 = 0xCF8; // Port to specify config address
const PCI_CONFIG_DATA: u16 = 0xCFC; // Port to read/write config data

/// Read a 32-bit value from PCI configuration space (internal, unsafe).
///
/// # How PCI config access works
/// PCI devices are accessed by writing a special address to 0xCF8, then reading/writing data at 0xCFC.
/// The address encodes the bus, device, function, and register offset.
///
/// # Safety
/// The caller must ensure that the provided bus, device, function, and offset values
/// are valid and that accessing the PCI configuration space in this way is safe on the
/// target hardware. Incorrect usage may cause undefined behavior or hardware faults.
pub unsafe fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    // Construct the address per PCI spec:
    // Bit 31: Enable bit
    // Bits 23-16: Bus number
    // Bits 15-11: Device number
    // Bits 10-8: Function number
    // Bits 7-2: Register offset (must be aligned to 4 bytes)
    // Bits 1-0: Always 0
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

/// Read a u8 from PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (byte granularity)
///
/// # Returns
/// The 8-bit value at the given offset
///
/// # Safety
/// See `pci_config_read`.
#[inline]
pub unsafe fn read_config_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    // PCI config reads are always 32-bit, so we shift/mask to get the correct byte
    let shift = (offset & 3) * 8;
    (unsafe { pci_config_read(bus, device, function, offset) } >> shift & 0xFF) as u8
}

/// Read a u16 from PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (byte granularity)
///
/// # Returns
/// The 16-bit value at the given offset
///
/// # Safety
/// See `pci_config_read`.
#[inline]
pub unsafe fn read_config_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    // PCI config reads are always 32-bit, so we may need to shift/mask
    let aligned = offset & !1; // Align to even offset (16-bit)
    let val = unsafe { pci_config_read(bus, device, function, aligned) };
    if (offset & 2) == 0 {
        (val & 0xFFFF) as u16
    } else {
        (val >> 16) as u16
    }
}

/// Read a u32 from PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (must be 4-byte aligned)
///
/// # Returns
/// The 32-bit value at the given offset
///
/// # Safety
/// See `pci_config_read`.
#[inline]
pub unsafe fn read_config_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    unsafe { pci_config_read(bus, device, function, offset) }
}

/// Write a 32-bit value to an I/O port.
///
/// # Safety
/// This is a low-level operation. Only use with valid port addresses.
unsafe fn outl(port: u16, val: u32) {
    // Inline assembly: write EAX to DX
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") val, options(nostack, preserves_flags));
    }
}

/// Read a 32-bit value from an I/O port.
///
/// # Safety
/// This is a low-level operation. Only use with valid port addresses.
unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    // Inline assembly: read EAX from DX
    unsafe {
        asm!("in eax, dx", out("eax") val, in("dx") port, options(nostack, preserves_flags));
    }
    val
}

/// Write a u8 to PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (byte granularity)
/// * `value` - The 8-bit value to write
///
/// # Safety
/// See `pci_config_write`.
#[inline]
pub unsafe fn write_config_u8(bus: u8, device: u8, function: u8, offset: u8, value: u8) {
    // PCI config writes are always 32-bit, so we must read-modify-write
    let shift = (offset & 3) * 8;
    let aligned = offset & !3; // Align to 4 bytes
    let mut orig = unsafe { pci_config_read(bus, device, function, aligned) };
    let mask = !(0xFFu32 << shift); // Mask out the target byte
    orig = (orig & mask) | ((value as u32) << shift); // Insert new value
    unsafe {
        pci_config_write(bus, device, function, aligned, orig);
    }
}

/// Write a u16 to PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (byte granularity)
/// * `value` - The 16-bit value to write
///
/// # Safety
/// See `pci_config_write`.
#[inline]
pub unsafe fn write_config_u16(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    // PCI config writes are always 32-bit, so we must read-modify-write
    let aligned = offset & !3; // Align to 4 bytes
    let shift = (offset & 2) * 8; // 0 or 16
    let mut orig = unsafe { pci_config_read(bus, device, function, aligned) };
    let mask = !(0xFFFFu32 << shift); // Mask out the target 16 bits
    orig = (orig & mask) | ((value as u32) << shift); // Insert new value
    unsafe {
        pci_config_write(bus, device, function, aligned, orig);
    }
}

/// Write a u32 to PCI configuration space.
///
/// # Arguments
/// * `bus`, `device`, `function` - PCI address
/// * `offset` - Register offset (must be 4-byte aligned)
/// * `value` - The 32-bit value to write
///
/// # Safety
/// See `pci_config_write`.
#[inline]
pub unsafe fn write_config_u32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    unsafe {
        pci_config_write(bus, device, function, offset, value);
    }
}

/// Internal: Write a 32-bit value to PCI configuration space (low-level).
///
/// # How it works
/// - Writes the address to 0xCF8
/// - Writes the value to 0xCFC
///
/// # Safety
/// The caller must ensure the address and value are valid for the hardware.
unsafe fn pci_config_write(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    // Write address to PCI_CONFIG_ADDRESS port
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
    }
    // Write value to PCI_CONFIG_DATA port
    unsafe {
        outl(PCI_CONFIG_DATA, value);
    }
}
