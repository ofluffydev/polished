//! ELF Loader Library
//!
//! This library provides functionality to load and execute a kernel or application from an ELF (Executable and Linkable Format) file.
//! It is designed for use in a UEFI bootloader context, where the kernel is loaded from disk and executed directly from its entry point address.
//!
//! # Overview
//!
//! - Reads an ELF file from disk (e.g., from an EFI system partition)
//! - Parses the ELF file and loads its segments into memory at the addresses specified by the ELF headers
//! - Allocates memory using UEFI services, respecting the segment permissions
//! - Copies segment data from the file into memory, zero-filling any uninitialized data (BSS)
//! - Returns the entry point address and a callable function pointer to start the loaded kernel
//!
//! # Usage
//!
//! Call [`load_kernel`] with the path to the ELF file. The function returns the entry point address and a function pointer you can call to transfer control to the loaded kernel.
//!
//! # Safety
//!
//! This code uses unsafe operations to copy memory and to transmute the entry point address into a function pointer. Ensure the ELF file is trusted and valid.
//!
//! # ELF Entry Point
//!
//! The entry point of an ELF file is a special address specified in the ELF header (the `e_entry` field). When the loader finishes loading all segments, it transfers control to this address to start execution of the program or kernel.
//!
//! ## How the Entry Point is Set
//!
//! - In Rust, you typically define the entry point function (e.g., `fn _start()`) and mark it with `#[no_mangle]` to prevent the compiler from renaming it.
//! - The linker script (e.g., `ENTRY(_start)`) tells the linker to set the ELF header's entry point to the address of your `_start` function.
//!
//! When this loader returns the entry point address and function pointer, it is using the value from the ELF header, which should match your `#[no_mangle]` entry function as set by your linker script.
//!
//! For more details, see the documentation for your linker and the `ENTRY()` directive in your linker script.

#![no_std]

#[cfg(feature = "uefi")]
use polished_files::read_file;
#[cfg(feature = "uefi")]
use uefi::boot::{self, AllocateType, MemoryType};
use xmas_elf::{ElfFile, program};

#[cfg(feature = "uefi")]
/// Loads a kernel from the specified ELF file path.
///
/// # Arguments
///
/// * `file_path` - The path to the ELF file to load (e.g., "\\EFI\\BOOT\\kernel").
///
/// # Returns
///
/// A tuple containing:
/// - The entry point address of the loaded kernel (as `usize`)
/// - A function pointer to the kernel's entry point (as `unsafe extern "C" fn() -> !`)
///
/// # How it works
///
/// 1. Reads the ELF file from disk into memory.
/// 2. Parses the ELF file and iterates over its program headers.
/// 3. For each loadable segment, allocates memory at the address requested by the ELF file.
/// 4. Copies the segment data from the file into the allocated memory, zero-filling any extra space (for BSS).
/// 5. Returns the entry point address and a function pointer to the entry point.
///
/// # Safety
///
/// The returned function pointer is only valid if the ELF file is well-formed and the memory was allocated and loaded correctly.
///
/// # Example
///
/// ```ignore
/// let (entry, kernel_entry) = load_kernel("\\EFI\\BOOT\\kernel");
/// // To start the kernel:
/// unsafe { kernel_entry() };
/// ```
pub fn load_kernel(file_path: &str) -> (usize, unsafe extern "C" fn() -> !) {
    // Log the file path being loaded
    log::info!("Loading kernel from ELF file: {file_path}");
    // Read the entire ELF file into memory
    let bytes = read_file(file_path).unwrap();
    // Parse the ELF file structure
    let elf = ElfFile::new(&bytes).expect("Failed to parse ELF file");

    // Iterate over each program header (segment) in the ELF file
    for ph in elf.program_iter() {
        let ph_type = ph.get_type().ok();
        log::info!("Found program header: {ph_type:?}");
        // Skip dynamic segments (not needed for kernel loading)
        if ph_type == Some(program::Type::Dynamic) {
            log::warn!("Skipping dynamic segment");
        }
        // Only process loadable segments
        if ph_type != Some(program::Type::Load) {
            continue;
        }

        // Get segment file offset, size in file, size in memory, and virtual address
        let file_offset = ph.offset() as usize;
        let file_size = ph.file_size() as usize;
        let mem_size = ph.mem_size() as usize;
        let virt_addr = ph.virtual_addr() as usize;

        log::info!(
            "Loading segment: file_offset=0x{file_offset:x}, file_size=0x{file_size:x}, mem_size=0x{mem_size:x}, virt_addr=0x{virt_addr:x}"
        );

        // Align the virtual address to a 4KiB page boundary
        let aligned_virt_addr = virt_addr & !0xFFF;
        let page_offset = virt_addr - aligned_virt_addr;
        let total_size = page_offset + mem_size;
        let num_pages = total_size.div_ceil(0x1000);

        // Choose memory type based on segment flags (executable or data)
        let mem_type = if ph.flags().is_execute() {
            MemoryType::LOADER_CODE
        } else {
            MemoryType::LOADER_DATA
        };

        log::info!(
            "Allocating {num_pages} pages at 0x{aligned_virt_addr:x} (mem_type: {mem_type:?})"
        );

        // Allocate memory at the requested virtual address
        let dest_ptr = boot::allocate_pages(
            AllocateType::Address(u64::try_from(aligned_virt_addr).unwrap()),
            mem_type,
            num_pages,
        )
        .expect("Failed to allocate pages")
        .as_ptr();

        unsafe {
            // Copy segment data from the ELF file into the allocated memory
            core::ptr::copy_nonoverlapping(
                bytes[file_offset..].as_ptr(),
                dest_ptr.add(page_offset),
                file_size,
            );

            // Zero-fill any remaining memory (for .bss or uninitialized data)
            if mem_size > file_size {
                core::ptr::write_bytes(
                    dest_ptr.add(page_offset + file_size),
                    0,
                    mem_size - file_size,
                );
            }
        }
        log::info!("Segment loaded at 0x{virt_addr:x}");
    }

    // Get the entry point address from the ELF header
    let entry_point = elf.header.pt2.entry_point() as usize;
    log::info!("Kernel entry point: 0x{entry_point:x}");
    // Convert the entry point address to a function pointer
    let kernel_entry: unsafe extern "C" fn() -> ! = unsafe { core::mem::transmute(entry_point) };

    (entry_point, kernel_entry)
}
