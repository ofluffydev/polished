//! PCI Lookup Tables and String Conversion Utilities
//!
//! PCI devices identify themselves using numeric codes ("magic numbers") for vendor, device, class, and subclass.
//! These codes are defined by the PCI specification and are not human-readable. To make sense of them during
//! debugging, logging, or display, these functions convert those codes into readable strings.
//!
//! For example, vendor ID 0x8086 means Intel, and class code 0x03 means Display Controller. Without these lookups,
//! you would just see numbers, which are not helpful for most people.
//!
//! In summary: PCI lookup tables/functions make hardware enumeration output understandable to humans.

/// Returns a human-readable string for a PCI vendor ID.
///
/// # Arguments
/// * `vendor_id` - The 16-bit PCI vendor ID from configuration space.
///
/// # Example
/// ```
/// let s = vendor_id_str(0x8086);
/// assert!(s.contains("Intel"));
/// ```
pub fn vendor_id_str(vendor_id: u16) -> &'static str {
    match vendor_id {
        0x8086 => "  vendor=0x8086 (Intel)",
        0x10DE => "  vendor=0x10DE (NVIDIA)",
        0x1234 => "  vendor=0x1234 (QEMU)",
        0x1AF4 => "  vendor=0x1AF4 (Red Hat / QEMU (VirtIO))",
        _ => "  vendor=unknown",
    }
}

/// Returns a human-readable string for a PCI class code.
///
/// # Arguments
/// * `class_code` - The 8-bit PCI class code from configuration space.
///
/// # Example
/// ```
/// let s = class_code_str(0x03);
/// assert!(s.contains("Display Controller"));
/// ```
pub fn class_code_str(class_code: u8) -> &'static str {
    match class_code {
        0x01 => "  class=0x01 (Mass Storage Controller)",
        0x02 => "  class=0x02 (Network Controller)",
        0x03 => "  class=0x03 (Display Controller)",
        0x06 => "  class=0x06 (Bridge Device)",
        _ => "  class=other",
    }
}

/// Returns a human-readable string for a PCI subclass code.
///
/// # Arguments
/// * `subclass` - The 8-bit PCI subclass code from configuration space.
///
/// # Example
/// ```
/// let s = subclass_str(0x80);
/// assert!(s.contains("Other"));
/// ```
pub fn subclass_str(subclass: u8) -> &'static str {
    match subclass {
        0x00 => "  subclass=0x00",
        0x01 => "  subclass=0x01",
        0x02 => "  subclass=0x02",
        0x03 => "  subclass=0x03",
        0x04 => "  subclass=0x04",
        0x05 => "  subclass=0x05",
        0x06 => "  subclass=0x06",
        0x80 => "  subclass=0x80 (Other)",
        _ => "  subclass=other",
    }
}
