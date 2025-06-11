//! Bootloader main library
//!
//! This module provides the main entry points for initializing the UEFI environment, loading the kernel,
//! setting up the framebuffer, and transferring control to the loaded kernel. It is designed to be used
//! as the core of a UEFI bootloader for a custom operating system.
//!
//! # What is UEFI?
//!
//! UEFI (Unified Extensible Firmware Interface) is a modern replacement for the legacy BIOS firmware found in PCs.
//! It provides a standard interface between the operating system and the platform firmware, allowing bootloaders
//! and OS kernels to interact with hardware in a consistent way. UEFI applications (like this bootloader) are loaded
//! and executed by the firmware before the OS starts. UEFI provides services for file access, graphics, input/output,
//! and more, making it easier to write portable bootloaders and OSes.
//!
//! # How does this bootloader use UEFI?
//!
//! This bootloader is a UEFI application. It uses UEFI services to:
//! - Load the kernel binary from disk (using UEFI file protocols)
//! - Set up a graphics framebuffer (using UEFI graphics protocols)
//! - Output text to the screen (using UEFI console protocols)
//! - Pass information (like framebuffer configuration) to the kernel
//! - Transfer control to the loaded kernel
//!
//! If you are new to UEFI, think of it as a set of helper functions provided by your computer's firmware
//! that let you interact with hardware and files before your OS is running.

#![no_std]

use core::arch::asm;

use log::info;
use polished_elf_loader::load_kernel;
use polished_graphics::framebuffer::{FramebufferInfo, initialize_framebuffer};
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::text::Output,
};

/// Boots the system by loading the kernel, initializing the framebuffer, and transferring control to the kernel.
///
/// # Arguments
/// * `kernel_path` - The UEFI path to the kernel binary to load. This is typically a path like
///   `\\efi\\boot\\kernel` on a FAT-formatted EFI system partition.
///
/// # How it works
/// 1. Loads the kernel binary from disk using UEFI file services.
/// 2. Initializes the graphics framebuffer using UEFI graphics protocols, so the kernel can draw to the screen.
/// 3. Passes the framebuffer configuration to the kernel as an argument.
/// 4. Uses inline assembly to jump to the kernel's entry point, transferring control to the OS.
///
/// # Safety
/// This function uses inline assembly to transfer control to the loaded kernel. After the call to the kernel's entry
/// point, the bootloader's execution is not guaranteed to continue. This is normal for bootloaders: once the OS starts,
/// the bootloader is no longer needed.
///
/// # UEFI for beginners
/// UEFI provides the services that make steps 1 and 2 possible. Without UEFI, you would have to write code to talk
/// directly to disk and graphics hardware, which is much more complex and less portable.
pub fn boot_system(kernel_path: &str) {
    // Load the kernel binary from the specified UEFI path. Returns the entry point address and a callable function pointer to the kernel's entry.
    let (entry_point, kernel_entry) = load_kernel(kernel_path);

    // Log the kernel's entry point address for debugging purposes.
    info!("Kernel entry point: 0x{:x}", kernel_entry as usize);

    // Log the address where we will jump to start the kernel.
    info!("Jumping to kernel entry point at 0x{entry_point:x}");

    // Initialize the framebuffer and retrieve its configuration info (resolution, address, etc.).
    let framebuffer_info = initialize_framebuffer();
    // Log the framebuffer information for debugging and diagnostics.
    info!("Framebuffer info: {framebuffer_info:?}");

    // Log again before transferring control to the kernel (redundant, but ensures visibility in logs).
    info!("Jumping to kernel entry point at 0x{entry_point:x}");

    unsafe {
        // Prepare a pointer to the framebuffer info struct to pass to the kernel.
        let fb_ptr = &framebuffer_info as *const FramebufferInfo;
        // Use inline assembly to set up the first argument (RDI) and call the kernel entry point.
        // This transfers control to the kernel, passing the framebuffer info pointer as an argument.
        asm!(
            "mov rdi, {0}",
            "call {1}",
            in(reg) fb_ptr,
            in(reg) kernel_entry,
        );
    }
}

/// Initializes the UEFI environment and clears the screen.
///
/// This function sets up the UEFI environment and clears the text output screen using the UEFI Output protocol.
///
/// # UEFI for beginners
/// UEFI provides a standard way to print text to the screen, regardless of the hardware. This function gets access
/// to the UEFI text output protocol and uses it to clear the screen, so any previous output is removed.
pub fn uefi_init() {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
}

/// Initializes the UEFI environment, clears the screen, and displays a greeting message.
///
/// # Arguments
/// * `greeting` - The message to display on the UEFI text output before clearing the screen again.
///
/// # UEFI for beginners
/// This function demonstrates how to print a message to the screen using UEFI services. It clears the screen,
/// prints the greeting, and then clears the screen again. This is useful for showing a welcome or status message
/// before the bootloader continues.
pub fn uefi_init_with_greeting(greeting: &str) {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
    info!("{greeting}");
    output.clear().expect("Failed to clear screen");
}
