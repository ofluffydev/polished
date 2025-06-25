// Example demo code for using the ustar archive in the kernel
use polished_files::ustar::tar_lookup;
use polished_pci::{probe_bar, scan_bus0_devices};
use polished_serial_logging::{info, warn};

extern crate alloc;
use alloc::format;

use polished_pci::error::PciError;

/// Demonstrates looking up and printing files from a ustar archive.
pub fn demo_ustar_archive(ustar_archive: &'static [u8]) {
    // Example usage of the ustar module
    let file = match tar_lookup(ustar_archive, "ustar-files/mymessage.txt") {
        Some(file) => file,
        None => {
            warn("File not found in archive: mytext.txt");
            (b"".as_slice(), 0)
        }
    };
    info(&format!("Found file: mytext.txt, size: {}", file.1));
    let file = match tar_lookup(ustar_archive, "ustar-files/hello_world.rs") {
        Some(file) => file,
        None => {
            warn("File not found in archive: hello_world.rs");
            (b"".as_slice(), 0)
        }
    };
    info(&format!("Found file: hello_world.rs, size: {}", file.1));

    // Print the contents of the file
    let file_contents = core::str::from_utf8(file.0).expect("Invalid UTF-8 in file");
    info(&format!("File contents:\n{file_contents}"));
}

pub fn demo_page_table() {
    // Allocate a page to test page table functionality
    unimplemented!("Page table demo not implemented yet");
}

pub fn pci_testdev_demo() {
    info("Running pci-testdev demo...");

    let devices = scan_bus0_devices().unwrap();

    // Log all found devices for debugging, using lookup functions for human-readable output
    for device in &devices {
        info(&alloc::format!(
            "PCI device: bus={}, device={}, function={}",
            device.bus,
            device.device,
            device.function
        ));
        info(polished_pci::lookup::vendor_id_str(device.vendor_id));
        info(&alloc::format!("  device_id=0x{:04x}", device.device_id));
        info(polished_pci::lookup::class_code_str(device.class));
        info(polished_pci::lookup::subclass_str(device.subclass));
    }

    let mut testdev0 = None;
    let mut testdev1 = None;
    for device in devices {
        if device.vendor_id == 0x1AF4 {
            info(&alloc::format!(
                "Found possible pci-testdev: device_id=0x{:04x}, bus={}, device={}, function={}",
                device.device_id,
                device.bus,
                device.device,
                device.function
            ));
            if testdev0.is_none() {
                testdev0 = Some(device);
            } else if testdev1.is_none() {
                testdev1 = Some(device);
            }
        }
    }

    let Some(device) = testdev0 else {
        warn("No pci-testdev device found.");
        return;
    };

    info(&alloc::format!(
        "Found pci-testdev at bus {}, device {}, function {}",
        device.bus,
        device.device,
        device.function
    ));

    for bar_index in 0..=1 {
        let bar_result =
            unsafe { probe_bar(device.bus, device.device, device.function, bar_index) };
        match bar_result {
            Err(PciError::DeviceNotFound) => {
                warn(&format!("BAR{bar_index} is unused"));
            }
            Err(e) => {
                warn(&format!("BAR{bar_index} error: {e:?}"));
            }
            Ok(bar) => {
                if bar.is_io {
                    info(&format!(
                        "BAR{bar_index} is I/O-mapped at 0x{:#x} (skipping MMIO demo for I/O BAR)",
                        bar.address
                    ));
                } else {
                    let base = bar.address;
                    info(&format!(
                        "BAR{} is MMIO at 0x{:x}, size={} bytes, prefetchable=N/A (skipping MMIO access)",
                        bar_index, base, bar.size
                    ));
                }
                // All MMIO mapping and pointer access is now skipped for both I/O and MMIO BARs
            }
        }
    }
}
