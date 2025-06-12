#![no_main]
#![no_std]

#[cfg(feature = "uefi")]
use polished_bootloader::{boot_system, uefi_init_with_greeting};
#[cfg(feature = "uefi")]
use uefi::prelude::*;

#[cfg(feature = "uefi")]
#[entry]
fn main() -> Status {
    uefi_init_with_greeting("Polished OS Bootloader online!");
    // Pass the kernel path as an argument
    boot_system("\\EFI\\BOOT\\kernel");

    Status::SUCCESS
}
