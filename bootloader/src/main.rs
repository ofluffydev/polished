#![no_main]
#![no_std]

extern crate alloc;

use core::arch::asm;

use elf_loader::load_kernel;
use graphics::framebuffer::{initialize_framebuffer, FramebufferInfo};
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
    let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\kernel");

    info!("Kernel entry point: 0x{:x}", kernel_entry as usize);

    info!("Jumping to kernel entry point at 0x{entry_point:x}");

    let framebuffer_info = initialize_framebuffer();
    info!("Framebuffer info: {framebuffer_info:?}");

    info!("Jumping to kernel entry point at 0x{entry_point:x}");

    unsafe {
        let fb_ptr = &framebuffer_info as *const FramebufferInfo;
        asm!(
            "mov rdi, {0}",
            in(reg) fb_ptr,
            options(nostack)
        );
        kernel_entry();
    }
}
