#![no_std]

// Library for parsing/loading ELF files in no_std environments.
// Implementation will be added later.

use files::read_file;
use uefi::boot::{self, AllocateType, MemoryType};
use xmas_elf::{ElfFile, program};

/// Loads a kernel from the specified ELF file path.
/// An example file path would be "\EFI\BOOT\kernel".
pub fn load_kernel(file_path: &str) -> (usize, unsafe extern "C" fn() -> !) {
    log::info!("Loading kernel from ELF file: {file_path}");
    let bytes = read_file(file_path).unwrap();
    let elf = ElfFile::new(&bytes).expect("Failed to parse ELF file");

    for ph in elf.program_iter() {
        let ph_type = ph.get_type().ok();
        log::info!("Found program header: {ph_type:?}");
        if ph_type == Some(program::Type::Dynamic) {
            log::warn!("Skipping dynamic segment");
        }
        if ph_type != Some(program::Type::Load) {
            continue;
        }

        let file_offset = ph.offset() as usize;
        let file_size = ph.file_size() as usize;
        let mem_size = ph.mem_size() as usize;
        let virt_addr = ph.virtual_addr() as usize;

        log::info!(
            "Loading segment: file_offset=0x{file_offset:x}, file_size=0x{file_size:x}, mem_size=0x{mem_size:x}, virt_addr=0x{virt_addr:x}"
        );

        let aligned_virt_addr = virt_addr & !0xFFF;
        let page_offset = virt_addr - aligned_virt_addr;
        let total_size = page_offset + mem_size;
        let num_pages = total_size.div_ceil(0x1000);

        let mem_type = if ph.flags().is_execute() {
            MemoryType::LOADER_CODE
        } else {
            MemoryType::LOADER_DATA
        };

        log::info!(
            "Allocating {num_pages} pages at 0x{aligned_virt_addr:x} (mem_type: {mem_type:?})"
        );

        // Allocate at the ELF's requested virtual address (no load base)
        let dest_ptr = boot::allocate_pages(
            AllocateType::Address(u64::try_from(aligned_virt_addr).unwrap()),
            mem_type,
            num_pages,
        )
        .expect("Failed to allocate pages")
        .as_ptr();

        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes[file_offset..].as_ptr(),
                dest_ptr.add(page_offset),
                file_size,
            );

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

    // Use the ELF's entry point directly
    let entry_point = elf.header.pt2.entry_point() as usize;
    log::info!("Kernel entry point: 0x{entry_point:x}");
    let kernel_entry: unsafe extern "C" fn() -> ! = unsafe { core::mem::transmute(entry_point) };

    (entry_point, kernel_entry)
}
