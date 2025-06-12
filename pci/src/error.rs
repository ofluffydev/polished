//! PCI Error Types
//!
//! This module defines error types for PCI operations, such as device access and enumeration.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciError {
    /// No device was found at the specified bus/device/function.
    DeviceNotFound,
    /// The requested offset is invalid for PCI config space.
    InvalidOffset,
    /// An I/O failure occurred during PCI access.
    IoFailure,
}
