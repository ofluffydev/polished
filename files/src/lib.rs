#![no_std]

#[cfg(feature = "uefi")]
pub mod uefi;

#[cfg(feature = "ext2")]
pub mod ext2;
