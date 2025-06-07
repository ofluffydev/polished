#![no_main]
#![no_std]

extern crate alloc;

use core::arch::asm;

use elf_loader::load_kernel;
use graphics::framebuffer::{FramebufferInfo, initialize_framebuffer};
use log::info;
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::text::Output,
};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
    info!("Polished OS Bootloader online!");
    boot::stall(2_000_000);
    output.clear().expect("Failed to clear screen");
    boot_system();

    Status::SUCCESS
}

pub fn boot_system() {
    // Load the kernel binary from the specified UEFI path. Returns the entry point address and a callable function pointer to the kernel's entry.
    let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\kernel");

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
