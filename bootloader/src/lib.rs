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

/*
    WARNING: Rust analyzer CONSTANTLY says numerous things in this file are unused.
    IT IS LYING.
*/

#![no_std]

#[cfg(feature = "uefi")]
extern crate uefi;

#[cfg(feature = "uefi")]
use core::arch::asm;

#[cfg(feature = "uefi")]
use log::info;
#[cfg(feature = "uefi")]
use polished_elf_loader::load_kernel;
#[cfg(feature = "uefi")]
use polished_graphics::framebuffer::FramebufferInfo;
#[cfg(feature = "uefi")]
use uefi::boot::exit_boot_services;
#[cfg(feature = "uefi")]
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::text::Output,
};

// static NOOP_LOGGER;

/// Boot information structure passed to the kernel by the bootloader.
///
/// This struct contains all the information the kernel needs to initialize itself after being loaded by the bootloader.
/// All addresses are physical addresses. The fields are designed to be extensible and compatible with C.
///
/// # Fields
/// - `memory_map_addr`: Physical address of the memory map provided by UEFI.
/// - `memory_map_entries`: Number of entries in the memory map.
/// - `initramfs_addr`: Physical address of the initramfs blob (if present).
/// - `initramfs_size`: Size of the initramfs blob in bytes.
/// - `cmdline_addr`: Physical address of the kernel command line string.
/// - `cmdline_len`: Length of the command line string (excluding null terminator).
/// - `framebuffer_addr`: Physical address of the framebuffer (if present).
/// - `framebuffer_width`: Width of the framebuffer in pixels.
/// - `framebuffer_height`: Height of the framebuffer in pixels.
/// - `framebuffer_pitch`: Number of bytes per scanline in the framebuffer.
/// - `framebuffer_bpp`: Bits per pixel in the framebuffer.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BootInfo {
    /// Physical address of the memory map.
    pub memory_map_addr: u64,
    /// Number of entries in the memory map.
    pub memory_map_entries: u64,

    /// Physical address of the initramfs blob.
    pub initramfs_addr: u64,
    /// Size of the initramfs blob in bytes.
    pub initramfs_size: u64,

    /// Physical address of the kernel command line string.
    pub cmdline_addr: u64,
    /// Length of the command line string (excluding null terminator).
    pub cmdline_len: u64,

    /// Optional framebuffer info.
    pub framebuffer_addr: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32,
    pub framebuffer_bpp: u8,

    pub usable_start_frame: u64,
    pub usable_end_frame: u64,
}

#[cfg(feature = "uefi")]
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
    use polished_allocators::frame::BumpFrameAllocator;
    use polished_serial_logging::kprint;
    use uefi::{boot::{self, memory_map, MemoryType}, mem::memory_map::MemoryMap};
    use x86_64::structures::paging::OffsetPageTable;
    info!("[boot] Searching for long free memory region");
    let mmap = memory_map(MemoryType::LOADER_DATA).unwrap();
    let memory_region = mmap.entries().find(|desc| desc.ty == MemoryType::CONVENTIONAL && desc.page_count >= 128).unwrap();
    info!("[boot] Starting page table and frame allocator initialization");
    let mut frame_alloc = unsafe { BumpFrameAllocator::new(memory_region.phys_start as usize, memory_region.phys_start as usize + 128 * 4096) };
    
    // Disable write protection so we can write to the page table
    // Enable NX pageflag for safety
    unsafe {
        use x86_64::registers::control::{Cr0Flags, EferFlags};

        x86_64::registers::control::Cr0::update(|x| x.remove(Cr0Flags::WRITE_PROTECT));
        x86_64::registers::control::Efer::update(|x| x.insert(EferFlags::NO_EXECUTE_ENABLE));
    }
    // Cr3 needs to contain a valid page table so this is safe
    // also virt_addr = phys_addr from uefi identity mappings
    let page_table = unsafe {
        use x86_64::structures::paging::PageTable;

        let (curr, _) = x86_64::registers::control::Cr3::read();
        &mut *(curr.start_address().as_u64() as *mut PageTable)
    };
    use x86_64::VirtAddr;
    // Offset is zero because everything is identity mapped
    let mut page_table = unsafe { OffsetPageTable::new(page_table, VirtAddr::zero()) };

    info!("[boot] Starting kernel load from path: {kernel_path}");
    let kernel_entry = load_kernel(kernel_path, &mut page_table, &mut frame_alloc);
    info!("[boot] Kernel load finished");
    info!("[boot] Kernel entry point: 0x{:x}", kernel_entry as usize);

    info!("[boot] Starting framebuffer initialization");
    let framebuffer_info = initialize_framebuffer();
    info!("[boot] Framebuffer initialization finished");
    info!("[boot] Framebuffer info: {framebuffer_info:?}");

    // Determine bits per pixel based on framebuffer format
    let (bpp, pitch) = match framebuffer_info.format {
        polished_graphics::framebuffer::FramebufferFormat::Rgb
        | polished_graphics::framebuffer::FramebufferFormat::Bgr
        | polished_graphics::framebuffer::FramebufferFormat::Bitmask => {
            (32u8, framebuffer_info.stride as u32 * 4)
        }
        polished_graphics::framebuffer::FramebufferFormat::BltOnly => (0u8, 0u32),
    };

    // NOTE: Paging setup and manipulation removed as requested.
    // You should implement your own paging setup here.

    log::set_max_level(log::LevelFilter::Off);
    kprint!("Falling back to kprint for early boot logging");

    // Construct BootInfo struct
    let boot_info = BootInfo {
        memory_map_addr: 0,    // TODO: Fill with real memory map address if needed
        memory_map_entries: 0, // TODO: Fill with real memory map entries if needed
        initramfs_addr: 0,     // TODO: Fill with real initramfs address if needed
        initramfs_size: 0,     // TODO: Fill with real initramfs size if needed
        cmdline_addr: 0,       // TODO: Fill with real cmdline address if needed
        cmdline_len: 0,        // TODO: Fill with real cmdline length if needed
        framebuffer_addr: framebuffer_info.address,
        framebuffer_width: framebuffer_info.width as u32,
        framebuffer_height: framebuffer_info.height as u32,
        framebuffer_pitch: pitch,
        framebuffer_bpp: bpp,
        // Set usable_start and usable_end for BootInfo
        usable_start_frame: 0x100000u64,
        usable_end_frame: 0, // TODO: Set real RAM size if needed
    };
    let boot_info_ptr = &boot_info as *const BootInfo;

    // Exit boot services
    kprint!("[boot] Calling exit_boot_services");
    let _mem_map = unsafe { exit_boot_services(None) };
    kprint!("[boot] exit_boot_services finished");

    // Jump to kernel entry point in higher half
    kprint!("[boot] Transferring control to kernel entry point");
    unsafe {
        asm!(
            "mov rdi, {}",
            "jmp {entry}",
            in(reg) boot_info_ptr,
            entry = in(reg) kernel_entry,
        );
    }
    kprint!("[boot] Kernel entry point call finished (should never return)");
}

// #[cfg(feature = "uefi")]
// fn find_usable_frame_range(mem_map: &mut MemoryMapIter) -> Option<(u64, u64)> {
//     let mut usable_start = u64::MAX;
//     let mut usable_end = 0;

//     for desc in mem_map {
//         if desc.ty == MemoryType::CONVENTIONAL {
//             let start = desc.phys_start;
//             let end = start + desc.page_count * 4096;
//             usable_start = usable_start.min(start);
//             usable_end = usable_end.max(end);
//         }
//     }

//     if usable_start < usable_end {
//         Some((usable_start, usable_end))
//     } else {
//         None
//     }
// }

#[cfg(feature = "uefi")]
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

#[cfg(feature = "uefi")]
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

/// Initialize the framebuffer using UEFI's Graphics Output Protocol (GOP).
///
/// # Returns
/// A `FramebufferInfo` struct describing the framebuffer's memory and display properties.
///
/// # Panics
/// This function will panic if the GOP protocol cannot be accessed (should only be used in UEFI environments).
#[cfg(feature = "uefi")]
pub fn initialize_framebuffer() -> FramebufferInfo {
    use polished_graphics::framebuffer::FramebufferFormat;
    use uefi::proto::console::gop::{self, GraphicsOutput};

    let gop_handle = get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop_protocol = open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let gop = gop_protocol.get_mut().unwrap();
    let mode_info = gop.current_mode_info();
    let resolution = mode_info.resolution();
    let stride = mode_info.stride();
    let pixel_format = mode_info.pixel_format();

    let mut gop_buffer = gop.frame_buffer();
    let gop_buffer_first_byte = gop_buffer.as_mut_ptr() as usize;

    info!("Framebuffer address: 0x{gop_buffer_first_byte:x}");
    info!("Framebuffer size: {} bytes", gop_buffer.size());

    FramebufferInfo {
        address: gop_buffer.as_mut_ptr() as u64,
        size: gop_buffer.size(),
        width: resolution.0,
        height: resolution.1,
        stride,
        format: match pixel_format {
            gop::PixelFormat::Rgb => FramebufferFormat::Rgb,
            gop::PixelFormat::Bgr => FramebufferFormat::Bgr,
            gop::PixelFormat::Bitmask => FramebufferFormat::Bitmask,
            gop::PixelFormat::BltOnly => FramebufferFormat::BltOnly,
        },
    }
}
