use polished_allocators::frame::{FrameAllocator, LockedFreeListFrameAllocator};
// Example demo code for using the ustar archive in the kernel
use polished_files::ustar::tar_lookup;
use polished_pci::{probe_bar, scan_bus0_devices};
use polished_serial_logging::{info, warn};

extern crate alloc;
use alloc::format;

use polished_pci::error::PciError;

use spin::Mutex;
use x86_64::structures::paging::PageTableFlags;

use lazy_static::lazy_static;

/// You must implement this to return the actual physical frame range your kernel can allocate from.
/// For example, from a UEFI bootloader memory map.
fn get_usable_memory_range() -> (u64, u64) {
    // This is a dummy example. You MUST replace this with actual memory info.
    // Return values are *physical addresses*, not byte sizes.
    let start_frame = 0x100_000; // Example: start at 1 MiB
    let end_frame = 0x400_000; // Example: end at 4 MiB
    (start_frame, end_frame)
}

lazy_static! {
    /// The global page allocator for the kernel, initialized with a free list allocator.
    pub static ref PAGE_ALLOCATOR: Mutex<LockedFreeListFrameAllocator> = {
        let (start, end) = get_usable_memory_range();

        // Sanity check
        assert!(start < end, "Frame allocator: invalid memory range");

        let allocator = unsafe { LockedFreeListFrameAllocator::new(start as usize, end as usize) };
        Mutex::new(allocator)
    };
}

/// Recursive PML4 is in entry 511
fn map_page(virt: u64, phys: u64, flags: PageTableFlags) {
    use x86_64::{PhysAddr, structures::paging::*};

    let r = 511;
    let indexes = [
        (virt >> 39) & 0x1FF,
        (virt >> 30) & 0x1FF,
        (virt >> 21) & 0x1FF,
        (virt >> 12) & 0x1FF,
    ];

    let mut table_addr = 0xFFFF_FFFF_FFFF_F000u64; // start at recursive PML4

    unsafe {
        for index in indexes[..3].iter() {
            let table: &mut PageTable = &mut *(table_addr as *mut PageTable);
            let entry = &mut table[*index as usize];
            if entry.is_unused() {
                // Allocate a page for the next level
                let new_table = PAGE_ALLOCATOR
                    .lock()
                    .allocate_frame()
                    .expect("Failed to allocate page for page table");
                entry.set_addr(
                    PhysAddr::new(new_table.start_address().try_into().unwrap()),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                );
            }
            table_addr = 0xFFFF_0000_0000_0000 | (r << 39) | (r << 30) | (r << 21) | (*index << 12);
        }

        let table: &mut PageTable = &mut *(table_addr as *mut PageTable);
        let entry = &mut table[indexes[3] as usize];
        entry.set_addr(PhysAddr::new(phys), flags);
    }
}

pub fn map_mmio_bar(virt_addr: u64, phys_addr: u64, size: u64) {
    let mut offset = 0;
    while offset < size {
        let vaddr = virt_addr + offset;
        let paddr = phys_addr + offset;

        map_page(
            vaddr,
            paddr,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE,
        );
        offset += 4096;
    }
}

/// Demonstrates looking up and printing files from a ustar archive.
pub fn demo_ustar_archive(ustar_archive: &'static [u8]) {
    // Example usage of the ustar module
    let file = match tar_lookup(ustar_archive, "ustar-files/mymessage.txt") {
        Some(file) => file,
        None => {
            warn("File not found in archive: mytext.txt");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: mytext.txt, size: {}", file.1));
    let file = match tar_lookup(ustar_archive, "ustar-files/hello_world.rs") {
        Some(file) => file,
        None => {
            warn("File not found in archive: hello_world.rs");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: hello_world.rs, size: {}", file.1));

    // Print the contents of the file
    let file_contents = core::str::from_utf8(file.0).expect("Invalid UTF-8 in file");
    info(&format!("File contents:\n{file_contents}"));
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
