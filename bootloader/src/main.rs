#![no_main]
#![no_std]

extern crate alloc;

use polished_bootloader::{boot_system, uefi_init_with_greeting};
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi_init_with_greeting("Polished OS Bootloader online!");
    // Pass the kernel path as an argument
    boot_system("\\EFI\\BOOT\\kernel");

    Status::SUCCESS
}
