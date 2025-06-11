//! # files
//!
//! A minimal library for reading files in `no_std` UEFI environments.
//!
//! This library provides a simple interface to load files from a UEFI filesystem using the `uefi` crate. It is designed for use in bootloaders or kernels where the standard library is unavailable. The main entry point is [`read_file`], which loads a file from the UEFI Simple File System protocol into a `Vec<u8>`.
//!
//! ## How it works
//!
//! - Uses UEFI boot services to access the loaded image's file system.
//! - Converts a UTF-8 path to a UEFI-compatible UTF-16 string (`CString16`).
//! - Opens the file system protocol and reads the file contents into a heap-allocated buffer.
//! - Returns the file data as a `Vec<u8>`, or an error if the operation fails.
//!
//! ## UEFI Context
//!
//! UEFI applications run in a pre-boot environment with access to firmware services. File access is provided via the Simple File System protocol, which exposes FAT-formatted volumes. This library abstracts the protocol details, allowing you to load files by path.

#![no_std]

// Library for loading files in no_std environments.

extern crate alloc;

#[cfg(feature = "uefi")]
use uefi::{
    CString16,
    boot::{self, ScopedProtocol},
    fs::FileSystem,
    proto::media::fs::SimpleFileSystem,
};

/// Reads the contents of a file from the UEFI file system into a `Vec<u8>`.
///
/// # Arguments
///
/// * `path` - The UTF-8 path to the file to load (e.g., "EFI/BOOT/bootx64.efi").
///
/// # Returns
///
/// * `Ok(Vec<u8>)` containing the file data if successful.
/// * `Err(FileSystemError)` if the file could not be read.
///
/// # How it works
///
/// 1. Converts the UTF-8 path to a UEFI-compatible UTF-16 string (`CString16`).
/// 2. Obtains the UEFI Simple File System protocol for the current image.
/// 3. Wraps the protocol in a [`FileSystem`] abstraction.
/// 4. Reads the file contents into a heap-allocated buffer (`Vec<u8>`).
///
/// # UEFI Details
///
/// UEFI file access is performed via the Simple File System protocol, which exposes FAT volumes to UEFI applications. This function uses the `uefi` crate's abstractions to safely open and read files from the firmware-provided file system.
///
/// # Panics
///
/// This function will panic if the path cannot be converted to UTF-16 or if the file system protocol cannot be opened. In production code, you may want to handle these errors more gracefully.
///
/// # Example
///
/// ```ignore
/// let data = read_file("EFI/BOOT/hello.txt")?;
/// // Use `data` as needed
/// ```
#[cfg(feature = "uefi")]
pub fn read_file(path: &str) -> uefi::fs::FileSystemResult<alloc::vec::Vec<u8>> {
    // Convert the UTF-8 path to a UEFI-compatible UTF-16 string
    let path: CString16 = CString16::try_from(path).unwrap();
    // Obtain the Simple File System protocol for the current image
    let fs: ScopedProtocol<SimpleFileSystem> =
        boot::get_image_file_system(boot::image_handle()).unwrap();
    // Wrap the protocol in a FileSystem abstraction
    let mut fs = FileSystem::new(fs);
    // Read the file contents into a Vec<u8>
    fs.read(path.as_ref())
}
