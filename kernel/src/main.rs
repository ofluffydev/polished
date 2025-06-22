#![no_std]
#![no_main]

extern crate alloc;

use polished_bootloader::{BootInfo, MEM_OFFSET};
use polished_memory as _; // Import the memory module for memset, memcpy, etc.
use polished_panic_handler as _;
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::OffsetPageTable;

use core::arch::{asm, naked_asm};
use polished_ps2::ps2_init;
use polished_serial_logging::info;

// Internal modules
mod allocator;
pub mod demos;
mod framebuffer_utils;
mod interrupts;

use crate::allocator::init_allocator;
use crate::framebuffer_utils::{clear_framebuffer, log_framebuffer_info};
use crate::interrupts::init_interrupts;

/// # Safety
/// This function is marked as naked and must only be called as the very first entry point
/// after boot. It must not make any stack-based accesses before the stack pointer is set.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn naked_start() {
    // Set up the stack pointer to the top of the stack
    naked_asm!(
        "cli",
        "lea rsp, STACK_TOP",
        "call {entry}",
        "2:",
        "cli",
        "hlt",
        "jmp 2b",
        entry = sym kernel_entry
    );
}

/// Unmap all identity-mapped pages in the lower half (0x0..0x00007fffffffffff)
pub fn clean_up_uefi_identity_mappings(table: &mut OffsetPageTable) {
    use polished_serial_logging::info;

    // Unmap all PML4 entries for lower half (0..256)
    for i in 0..256 {
        table.level_4_table_mut()[i].set_unused();
    }

    tlb::flush_all();
    info("[paging] Cleaned up UEFI identity mappings (lower half)");
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `boot_info_ptr` must be a valid pointer to a `BootInfo` structure, or null.
unsafe fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    // Initialize heap allocator
    init_allocator();
    let boot_info = unsafe { *boot_info_ptr }; // copy it because we cant guarrantee it will still be mapped once we clean out identity mappings
    let fb = crate::framebuffer_utils::make_framebuffer_info(&boot_info);

    info("Hello from the kernel!");

    // Clean up uefi identity mapping since we are offset mapped now
    let (table, _) = Cr3::read();
    let mut page_table = unsafe { OffsetPageTable::new(&mut *((table.start_address().as_u64() + MEM_OFFSET.as_u64()) as *mut _), MEM_OFFSET) };
    clean_up_uefi_identity_mappings(&mut page_table);

    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");

    // Set up interrupts and PS/2
    init_interrupts();
    ps2_init();

    // Framebuffer info and clear
    log_framebuffer_info(&fb);
    clear_framebuffer(&fb);

    // Enable CPU interrupts
    // x86_64::instructions::interrupts::enable();

    // Assembly code to allow interrupts
    unsafe {
        asm!("sti");
    }

    // PCI device scan
    // pci_enumeration_demo();

    // QEMU pci-testdev demo
    crate::demos::pci_testdev_demo();
    info("PCI test device demo completed");

    // Demo: extract and print files from ustar archive
    let ustar_archive = include_bytes!("../../archive.tar");
    crate::demos::demo_ustar_archive(ustar_archive);

    // Main kernel loop
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    loop {
        unsafe {
            asm!("pause; hlt");
        } // Use PAUSE before HLT for better power efficiency
    }
}
