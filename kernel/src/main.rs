#![no_std]
#![no_main]

extern crate alloc;

use polished_bootloader::BootInfo;
use polished_memory as _; // Import the memory module for memset, memcpy, etc.
use polished_panic_handler as _;

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
pub fn cleanup_uefi_identity_mappings() {
    use polished_serial_logging::info;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{PageTable, PageTableFlags};

    // Use physmap to access PML4
    let pml4_phys = crate::demos::get_pml4_phys();
    let pml4_virt = 0xFFFF_8000_0000_0000u64 + pml4_phys;
    let pml4: &mut PageTable = unsafe { &mut *(pml4_virt as *mut PageTable) };

    // Unmap all PML4 entries for lower half (0..256)
    for i in 0..256 {
        if pml4[i].flags().contains(PageTableFlags::PRESENT) {
            pml4[i].set_unused();
        }
    }
    // Flush TLB for the entire lower half
    for i in 0..256 {
        let virt = (i as u64) << 39;
        x86_64::instructions::tlb::flush(VirtAddr::new(virt));
    }
    info("[paging] Cleaned up UEFI identity mappings (lower half)");
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `boot_info_ptr` must be a valid pointer to a `BootInfo` structure, or null.
unsafe fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    // Initialize heap allocator
    init_allocator();
    let boot_info = unsafe { &*boot_info_ptr };
    let fb = crate::framebuffer_utils::make_framebuffer_info(boot_info);

    info("Hello from the kernel!");
    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");

    // Set the PML4 physical address for physmap page table access
    // set_boot_pml4_phys(boot_info.pml4_phys);

    // Set up interrupts and PS/2
    init_interrupts();
    ps2_init();

    // Framebuffer info and clear
    log_framebuffer_info(&fb);
    clear_framebuffer(&fb);

    // Clean up UEFI identity mappings
    // cleanup_uefi_identity_mappings();

    // Enable CPU interrupts
    // x86_64::instructions::interrupts::enable();

    // Assembly code to allow interrupts
    unsafe {
        asm!("sti");
    }

    // PCI device scan
    // pci_enumeration_demo();

    // stop interrupts for a minute
    // without_interrupts(|| {
    //     // Paging test
    //     info("Running paging test...");
    //     test_page_mapping();
    //     info("Paging test completed successfully");
    // });

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

// /// Set the PML4 physical address for physmap page table access
// fn set_boot_pml4_phys(pml4_phys: u64) {
//     unsafe extern "C" {
//         static mut BOOT_PML4_PHYS: u64;
//     }
//     unsafe {
//         BOOT_PML4_PHYS = pml4_phys;
//     }
// }

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.boot_pml4_phys")]
pub static mut BOOT_PML4_PHYS: u64 = 0;
