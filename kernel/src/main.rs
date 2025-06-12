#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use polished_bootloader::BootInfo;
use polished_memory as _; // Import the memory module for memset, memcpy, etc.
use polished_panic_handler as _; // Import the panic handler.

use core::arch::{asm, naked_asm};
use polished_pci::{pci_config_read, print_pci_device};
use polished_ps2::ps2_init;
use polished_serial_logging::info;

mod allocator;
pub mod demos;
mod framebuffer_utils;
mod interrupts;

use crate::allocator::init_allocator;
use crate::framebuffer_utils::{clear_framebuffer, log_framebuffer_info};
use crate::interrupts::init_interrupts;

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn naked_start() {
    // Set up the stack pointer to the top of the stack
    naked_asm!(
        "cli",
        "lea rsp, STACK_TOP",
        "call kernel_entry",
        "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    init_allocator();
    let boot_info = unsafe { &*boot_info_ptr };
    let fb = crate::framebuffer_utils::make_framebuffer_info(boot_info);
    info("Hello from the kernel!");
    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");
    init_interrupts();
    ps2_init();
    log_framebuffer_info(&fb);
    clear_framebuffer(&fb);
    x86_64::instructions::interrupts::enable();
    // Only disable the PIC after confirming interrupts work, or comment out for now
    // info("Disabling legacy PIC...");
    // disable_pic();
    // info("Legacy PIC disabled");
    // simulate_divide_by_zero();

    let ustar_archive = include_bytes!("../../archive.tar");
    crate::demos::demo_ustar_archive(ustar_archive);

    let found_virtio = (0u8..32).find_map(|device| {
        let vendor_id = unsafe { pci_config_read(0, device, 0, 0) & 0xFFFF } as u16;
        if vendor_id == 0xFFFF {
            return None;
        }
        let device_id = ((unsafe { crate::pci_config_read(0, device, 0, 0) }) >> 16) as u16;
        let class = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 24) as u8;
        let subclass = ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 16) as u8;
        print_pci_device(0, device, vendor_id, device_id, class, subclass);
        if vendor_id == 0x1AF4 && device_id == 0x1001 && class == 0x01 && subclass == 0x00 {
            Some(polished_pci::PciDevice {
                bus: 0,
                device,
                function: 0,
                vendor_id,
                device_id,
                class,
                subclass,
                prog_if: ((unsafe { crate::pci_config_read(0, device, 0, 8) }) >> 8) as u8,
                header_type: ((unsafe { crate::pci_config_read(0, device, 0, 0xC) }) >> 16) as u8,
            })
        } else {
            None
        }
    });

    if let Some(virtio_device) = found_virtio {
        info(&format!("Found VirtIO device: {virtio_device:?}"));
        let virtio_device = polished_virtio::VirtioDevice::from(virtio_device);
        // Just log the info about the VirtIO device
        info(&format!("VirtIO device info: {virtio_device:?}"));

        // Initialize the VirtIO device
        if let Err(e) = virtio_device.init() {
            info(&format!("Failed to initialize VirtIO device: {:?}", e));
        } else {
            info("VirtIO device initialized successfully");
        }
    } else {
        info("No VirtIO device found");
    }

    // Loop forever to keep the kernel running
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    loop {
        unsafe { asm!("pause; hlt") }; // Use PAUSE before HLT for better power efficiency
    }

    // panic!("Kernel halted");
}
