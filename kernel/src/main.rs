#![no_std]
#![no_main]

extern crate alloc;

use polished_bootloader::BootInfo;
use polished_memory as _; // Import the memory module for memset, memcpy, etc.
use polished_panic_handler as _;
use polished_pci::pci_enumeration_demo; // Import the panic handler.

use core::arch::{asm, naked_asm};
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

    // Scan and print all PCI devices using polished_pci
    pci_enumeration_demo();

    let ustar_archive = include_bytes!("../../archive.tar");
    crate::demos::demo_ustar_archive(ustar_archive);

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
