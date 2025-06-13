//! # Virtio Device Abstraction Library
//!
//! This library provides a safe and ergonomic interface for interacting with Virtio devices
//! using port-mapped I/O (PIO) in a no_std environment, such as a hobby operating system kernel.
//!
//! ## What is Virtio?
//!
//! Virtio is a standardized interface for virtual devices, commonly used in virtual machines
//! (like QEMU or KVM) to provide fast, efficient access to hardware-like devices such as
//! network cards, block devices, and more. Virtio devices are designed to be simple to implement
//! and easy to use from an OS kernel.
//!
//! ## What does this library do?
//!
//! This library defines a `VirtioDevice` struct and related enums to help you:
//! - Read and write to Virtio device registers using port I/O
//! - Manage device status and features
//! - Select and configure device queues
//! - Safely reset and initialize Virtio devices
//!
//! All methods are marked `unsafe` where direct hardware access is performed, and detailed
//! documentation is provided to help beginners understand the risks and requirements of each operation.
//!
//! ## Example Usage
//!
//! ```no_run
//! use polished_virtio::VirtioDevice;
//!
//! // Create a Virtio device abstraction for a device at bus 0, device 5, function 0, I/O base 0xC000
//! let virtio = VirtioDevice::new(0, 5, 0, 0xC000);
//! // Reset the device (unsafe: must ensure device is ready to be reset)
//! unsafe { virtio.reset(); }
//! ```

#![no_std]

use core::arch::asm;
use polished_pci::PciDevice;

/// Represents a Virtio device discovered on the PCI bus.
///
/// This struct holds the PCI bus location and the I/O base address (BAR) for the device.
/// The I/O base is used for all register accesses via port-mapped I/O.
#[derive(Debug)]
pub struct VirtioDevice {
    /// PCI bus number where the device is located.
    pub bus: u8,
    /// PCI device number.
    pub device: u8,
    /// PCI function number.
    pub function: u8,
    /// I/O BAR base address for port-mapped register access.
    pub io_base: u16, // I/O BAR base address
}

/// Enum of all standard Virtio device register offsets (in bytes).
///
/// These offsets are added to the device's I/O base to access specific registers.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum VirtioRegister {
    /// Magic value (should be 0x74726976 for Virtio devices)
    MagicValue = 0x00,
    /// Version (should be 0x2 for modern Virtio)
    Version = 0x04,
    /// Device ID (identifies the type of Virtio device)
    DeviceId = 0x08,
    /// Vendor ID (should be 0x1AF4 for Virtio)
    VendorId = 0x0C,
    /// Device feature bits (read-only)
    DeviceFeatures = 0x10,
    /// Driver feature bits (write-only)
    DriverFeatures = 0x20,
    /// Queue select register
    QueueSelect = 0x30,
    /// Queue size register
    QueueSize = 0x34,
    /// Queue address register (physical address of the queue)
    QueueAddr = 0x40,
    /// Device status register
    Status = 0x70,
}

/// Bitflags for the Virtio device status register.
///
/// These are used to communicate the current state of the driver to the device.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum VirtioStatus {
    /// Driver has noticed the device.
    Acknowledge = 0x01,
    /// Driver knows how to drive the device.
    Driver = 0x02,
    /// Driver is set up and ready.
    DriverOk = 0x04,
    /// Driver has accepted the device's feature set.
    FeaturesOk = 0x08,
}

impl VirtioDevice {
    /// Create a new VirtioDevice abstraction for a device at the given PCI location and I/O base.
    ///
    /// # Arguments
    /// * `bus` - PCI bus number
    /// * `device` - PCI device number
    /// * `function` - PCI function number
    /// * `io_base` - I/O BAR base address for port-mapped register access
    pub fn new(bus: u8, device: u8, function: u8, io_base: u16) -> Self {
        Self {
            bus,
            device,
            function,
            io_base,
        }
    }

    /// Reads a 32-bit value from the device at the given offset (in bytes from the I/O base).
    ///
    /// # Arguments
    /// * `offset` - The offset (in bytes) from the device's I/O base address.
    ///
    /// # Returns
    /// The 32-bit value read from the device register.
    ///
    /// # Safety
    ///
    /// This function performs a direct hardware I/O read. The caller must ensure:
    /// - The offset is valid for the device.
    /// - The device is present and ready.
    /// - Reading from this port will not cause undefined behavior.
    ///
    /// In OS development, invalid I/O operations can crash the system or corrupt device state.
    pub unsafe fn read_u32(&self, offset: u16) -> u32 {
        let port = self.io_base + offset;
        let val: u32;
        unsafe {
            asm!(
                "in eax, dx",
                out("eax") val,
                in("dx") port,
                options(nomem, nostack, preserves_flags)
            );
        }
        val
    }

    /// Writes a 32-bit value to the device at the given offset (in bytes from the I/O base).
    ///
    /// # Arguments
    /// * `offset` - The offset (in bytes) from the device's I/O base address.
    /// * `val` - The 32-bit value to write to the device register.
    ///
    /// # Safety
    ///
    /// This function performs a direct hardware I/O write. The caller must ensure:
    /// - The offset is valid for the device.
    /// - The device is present and ready.
    /// - Writing to this port will not cause undefined behavior.
    ///
    /// Incorrect writes can crash the system or put the device in an invalid state.
    pub unsafe fn write_u32(&self, offset: u16, val: u32) {
        let port = self.io_base + offset;
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") port,
                in("eax") val,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    /// Reads a 32-bit value from the device at the given register.
    ///
    /// # Arguments
    /// * `reg` - The VirtioRegister enum value representing the register to read.
    ///
    /// # Returns
    /// The 32-bit value read from the register.
    ///
    /// # Safety
    ///
    /// Same requirements as [`read_u32`].
    pub unsafe fn read_u32_reg(&self, reg: VirtioRegister) -> u32 {
        unsafe { self.read_u32(reg as u16) }
    }

    /// Writes a 32-bit value to the device at the given register.
    ///
    /// # Arguments
    /// * `reg` - The VirtioRegister enum value representing the register to write.
    /// * `val` - The 32-bit value to write.
    ///
    /// # Safety
    ///
    /// Same requirements as [`write_u32`].
    pub unsafe fn write_u32_reg(&self, reg: VirtioRegister, val: u32) {
        unsafe { self.write_u32(reg as u16, val) }
    }

    /// Resets the device by writing 0 to the status register.
    ///
    /// This is usually the first step before initializing a Virtio device.
    ///
    /// # Safety
    ///
    /// Only call this if you are sure the device is ready to be reset. Resetting at the wrong time
    /// can cause data loss or undefined behavior.
    pub unsafe fn reset(&self) {
        unsafe { self.write_u32_reg(VirtioRegister::Status, 0) };
    }

    /// Sets the device status register to the given value.
    ///
    /// # Arguments
    /// * `status` - The new status value (bitflags from VirtioStatus).
    ///
    /// # Safety
    ///
    /// Only set status values that are valid for the device's current state. Setting an invalid
    /// status can break the device or cause undefined behavior.
    pub unsafe fn set_status(&self, status: u32) {
        unsafe { self.write_u32_reg(VirtioRegister::Status, status) };
    }

    /// Reads the device status register.
    ///
    /// # Returns
    /// The current value of the status register.
    ///
    /// # Safety
    ///
    /// Only read the status register when the device is present and ready.
    pub unsafe fn get_status(&self) -> u32 {
        unsafe { self.read_u32_reg(VirtioRegister::Status) }
    }

    /// Acknowledges the device by setting the status register to ACKNOWLEDGE.
    ///
    /// This is the first step in the Virtio initialization sequence.
    ///
    /// # Safety
    ///
    /// Only call this if the device is present and ready to be acknowledged.
    pub unsafe fn acknowledge(&self) {
        unsafe { self.set_status(VirtioStatus::Acknowledge as u32) };
    }

    /// Sets the DRIVER status bit in the device status register.
    ///
    /// This tells the device that the driver knows how to operate it.
    ///
    /// # Safety
    ///
    /// Only call this after acknowledging the device.
    pub unsafe fn driver(&self) {
        unsafe { self.set_status(self.get_status() | VirtioStatus::Driver as u32) };
    }

    /// Reads the device's feature bits.
    ///
    /// # Returns
    /// The feature bits supported by the device.
    ///
    /// # Safety
    ///
    /// Only call this after the device is acknowledged and ready.
    pub unsafe fn features(&self) -> u32 {
        unsafe { self.read_u32_reg(VirtioRegister::DeviceFeatures) }
    }

    /// Writes the given feature bits to the device.
    ///
    /// # Arguments
    /// * `features` - The feature bits supported by the driver.
    ///
    /// # Safety
    ///
    /// Only call this after reading the device's features and negotiating them.
    pub unsafe fn write_features(&self, features: u32) {
        unsafe { self.write_u32_reg(VirtioRegister::DriverFeatures, features) };
    }

    /// Sets the FEATURES_OK status bit in the device status register.
    ///
    /// This tells the device that the driver has accepted the feature set.
    ///
    /// # Safety
    ///
    /// Only call this after writing the negotiated features.
    pub unsafe fn features_ok(&self) {
        unsafe { self.set_status(self.get_status() | VirtioStatus::FeaturesOk as u32) };
    }

    /// Sets the DRIVER_OK status bit in the device status register.
    ///
    /// This tells the device that the driver is fully set up and ready.
    ///
    /// # Safety
    ///
    /// Only call this after all initialization steps are complete.
    pub unsafe fn driver_ok(&self) {
        unsafe { self.set_status(self.get_status() | VirtioStatus::DriverOk as u32) };
    }

    /// Selects the queue with the given index for the device.
    ///
    /// # Arguments
    /// * `queue_index` - The index of the queue to select.
    ///
    /// # Safety
    ///
    /// Only select valid queue indices for the device. Selecting an invalid queue can cause undefined behavior.
    pub unsafe fn select_queue(&self, queue_index: u16) {
        unsafe { self.write_u32_reg(VirtioRegister::QueueSelect, queue_index as u32) };
    }

    /// Reads the size of the currently selected queue.
    ///
    /// # Returns
    /// The size of the selected queue.
    ///
    /// # Safety
    ///
    /// Only call this after selecting a valid queue.
    pub unsafe fn get_queue_size(&self) -> u16 {
        unsafe { self.read_u32_reg(VirtioRegister::QueueSize) as u16 }
    }

    /// Sets the physical address of the queue for the device.
    ///
    /// # Arguments
    /// * `phys_addr` - The physical address of the queue (must be 4K-aligned).
    ///
    /// # Safety
    ///
    /// Only set the queue address after allocating and preparing the queue in memory.
    pub unsafe fn set_queue_addr(&self, phys_addr: u64) {
        unsafe { self.write_u32_reg(VirtioRegister::QueueAddr, (phys_addr >> 12) as u32) };
    }

    /// Verifies the device identity by checking specific registers for expected values.
    ///
    /// This is useful for confirming that a device is a valid Virtio device before initializing it.
    ///
    /// # Returns
    /// `true` if the device matches the expected Virtio values, `false` otherwise.
    ///
    /// # Safety
    ///
    /// Only call this if you are sure the device is present and accessible.
    pub unsafe fn verify_identity(&self) -> bool {
        unsafe {
            self.read_u32_reg(VirtioRegister::MagicValue) == 0x74726976
                && self.read_u32_reg(VirtioRegister::Version) == 0x2
                && self.read_u32_reg(VirtioRegister::DeviceId) == 0x2
                && self.read_u32_reg(VirtioRegister::VendorId) == 0x1AF4
        }
    }

    /// Initializes the Virtio device using the standard initialization sequence.
    ///
    /// This method performs the following steps:
    /// 1. Resets the device
    /// 2. Acknowledges the device
    /// 3. Sets the DRIVER status
    /// 4. Reads and writes back the device features (accepts all features)
    /// 5. Sets FEATURES_OK and checks if the device accepted it
    /// 6. Sets DRIVER_OK
    ///
    /// # Returns
    /// * `Ok(())` if initialization succeeded
    /// * `Err(&'static str)` if initialization failed at any step
    pub fn init(&self) -> Result<(), &'static str> {
        unsafe {
            self.reset();
            self.acknowledge();
            self.driver();
            let features = self.features();
            self.write_features(features);
            self.features_ok();
            // Check if device accepted FEATURES_OK
            let status = self.get_status();
            if status & (VirtioStatus::FeaturesOk as u32) == 0 {
                return Err("Device did not accept FEATURES_OK");
            }
            self.driver_ok();
        }
        Ok(())
    }

    /// Sets up a Virtio queue at the given index.
    ///
    /// # Arguments
    /// * `index` - The queue index to set up.
    ///
    /// # Returns
    /// * `Ok(())` if the queue was set up successfully
    /// * `Err(&'static str)` if allocation or setup failed
    ///
    /// # Safety
    /// This function performs direct hardware access and assumes the device is properly initialized.
    pub fn setup_queue(&self, index: u16) -> Result<(), &'static str> {
        unsafe {
            self.select_queue(index);
            let size = self.get_queue_size();
            if size == 0 {
                return Err("Queue size is 0");
            }

            // Here you'd allocate and zero-initialize a properly aligned buffer
            // The buffer must be 4K-aligned and large enough for the queue
            // This is a placeholder; you must provide your own allocator
            let addr = allocate_virtqueue(size).ok_or("Failed to allocate queue")?;

            self.set_queue_addr(addr);
        }
        Ok(())
    }
}

/// Allocates a 4K-aligned, zero-initialized buffer for a Virtqueue of the given size (number of entries).
/// Returns the physical address of the buffer, or None on failure.
///
/// This is a simple placeholder implementation using a static buffer for demonstration only.
/// In a real OS, you should use your kernel's page allocator and return the physical address.
pub fn allocate_virtqueue(size: u16) -> Option<u64> {
    const MAX_QUEUE: usize = 256; // adjust as needed
    static mut VIRTQUEUE_BUF: [u8; 4096 * 4] = [0; 4096 * 4]; // 16 KiB static buffer
    if size as usize > MAX_QUEUE {
        return None;
    }
    // SAFETY: We only return the address, not a reference, so this is allowed.
    let ptr = core::ptr::addr_of_mut!(VIRTQUEUE_BUF) as usize;
    let aligned = (ptr + 0xFFF) & !0xFFF;
    Some(aligned as u64)
}

impl From<PciDevice> for VirtioDevice {
    /// Converts a PCI device into a VirtioDevice, panicking if the device is not a valid Virtio device.
    ///
    /// This implementation expects the PCI device to be a legacy Virtio device using port-mapped I/O (PIO).
    fn from(pci_dev: PciDevice) -> Self {
        use polished_pci::get_bars;
        // Virtio legacy devices: vendor_id = 0x1AF4
        assert_eq!(
            pci_dev.vendor_id, 0x1AF4,
            "Not a Virtio device: vendor_id != 0x1AF4"
        );
        // Get all BARs for this device (unsafe: direct PCI access)
        let bars = unsafe { get_bars(pci_dev.bus, pci_dev.device, pci_dev.function) }
            .expect("Failed to read PCI BARs");
        // Find the first I/O BAR (legacy Virtio uses I/O BAR)
        let io_bar = bars
            .iter()
            .find(|bar| bar.is_io)
            .expect("No I/O BAR found for Virtio device");
        let io_base = io_bar.address as u16;
        Self {
            bus: pci_dev.bus,
            device: pci_dev.device,
            function: pci_dev.function,
            io_base,
        }
    }
}
