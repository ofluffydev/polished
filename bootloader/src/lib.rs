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
use uefi::{boot::MemoryType, mem::memory_map::MemoryMapIter};
#[cfg(feature = "uefi")]
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::text::Output,
};

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
    pub kernel_entry_addr: u64,

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
    // Load the kernel binary from the specified UEFI path. Returns the entry point address and a callable function pointer to the kernel's entry.
    use core::ptr::NonNull;

    use uefi::{
        boot::{AllocateType, MemoryType, exit_boot_services},
        mem::memory_map::MemoryMap,
    };
    use x86_64::{PhysAddr, registers::control::Cr3, structures::paging::PhysFrame};
    let (entry_point, kernel_entry) = load_kernel(kernel_path);

    // Log the kernel's entry point address for debugging purposes.
    info!("Kernel entry point: 0x{:x}", kernel_entry as usize);

    // Log the address where we will jump to start the kernel.
    info!("Jumping to kernel entry point at 0x{entry_point:x}");

    // Initialize the framebuffer and retrieve its configuration info (resolution, address, etc.).
    let framebuffer_info = initialize_framebuffer();
    info!("Framebuffer info: {framebuffer_info:?}");

    // Determine bits per pixel based on framebuffer format (assume 32bpp for Rgb/Bgr/Bitmask, 0 for BltOnly)
    let (bpp, pitch) = match framebuffer_info.format {
        polished_graphics::framebuffer::FramebufferFormat::Rgb
        | polished_graphics::framebuffer::FramebufferFormat::Bgr
        | polished_graphics::framebuffer::FramebufferFormat::Bitmask => {
            (32u8, framebuffer_info.stride as u32 * 4)
        }
        polished_graphics::framebuffer::FramebufferFormat::BltOnly => (0u8, 0u32),
    };

    // How many 4-KiB pages do you need? 4 pages → 16-KiB total.
    const PAGE_COUNT: usize = 4;

    // This returns a NonNull<u8> whose address is the physical base of the region.
    let pages: NonNull<u8> =
        uefi::boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, PAGE_COUNT)
            .expect("Failed to allocate pages");

    // Convert it to a physical base address:
    let phys_start: u64 = pages.as_ptr() as u64;

    // Zero the 16-KiB you just got:
    unsafe {
        core::ptr::write_bytes(pages.as_ptr(), 0, PAGE_COUNT * 4096);
    }

    // A bare-bones page-table page (4096 bytes, 512 entries * u64 = 4096)
    #[repr(align(4096))]
    struct PageTable([u64; 512]);

    // --- 1 GiB huge-page identity map for 0-512 GiB ---
    let pdpt_phys = phys_start + 0x1000;
    unsafe {
        core::ptr::write_bytes(pdpt_phys as *mut u8, 0, 4096);
    }
    let pdpt: &mut PageTable = unsafe { &mut *(pdpt_phys as *mut PageTable) };
    const PDPT_FLAGS: u64 = 0b10000011; // Present | Writable | PS (1 GiB)
    for i in 0..512 {
        let addr = (i as u64) << 30; // i * 1 GiB
        pdpt.0[i] = addr | PDPT_FLAGS;
    }
    let pml4: &mut PageTable = unsafe { &mut *(phys_start as *mut PageTable) };
    pml4.0[0] = pdpt_phys | 0b11; // Present | Writable
    // Set up recursive mapping: PML4[511] points to itself
    pml4.0[511] = phys_start | 0b11; // Present | Writable

    // Build a PhysFrame from your PML4's physical base:
    let frame = PhysFrame::from_start_address(PhysAddr::new(phys_start)).expect("page-aligned");
    let (_, old_flags) = Cr3::read();

    // This makes your new tables live:
    unsafe { Cr3::write(frame, old_flags) };

    // --- Probe the kernel mapping before exit_boot_services ---
    const KERNEL_VIRT: u64 = 0x1400_00000;
    unsafe {
        let probe: *const u64 = KERNEL_VIRT as *const u64;
        let _ = core::ptr::read_volatile(probe);
    }

    // Now pray that paging isn't broke and boot.
    let mem_map = unsafe { exit_boot_services(None) };
    let mut entries = mem_map.entries();
    let (usable_start, usable_end) =
        find_usable_frame_range(&mut entries).expect("No usable memory regions found");

    // Construct BootInfo struct to pass to the kernel
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

        // We can have some fun later with this address.
        kernel_entry_addr: entry_point as u64,
        usable_start_frame: usable_start,
        usable_end_frame: usable_end,
    };

    let boot_info_ptr = &boot_info as *const BootInfo;

    unsafe {
        asm!(
            "mov rdi, {0}",
            "call {1}",
            in(reg) boot_info_ptr,
            in(reg) kernel_entry,
        );
    }
}

#[cfg(feature = "uefi")]
fn find_usable_frame_range(mem_map: &mut MemoryMapIter) -> Option<(u64, u64)> {
    let mut usable_start = u64::MAX;
    let mut usable_end = 0;

    for desc in mem_map {
        if desc.ty == MemoryType::CONVENTIONAL {
            let start = desc.phys_start;
            let end = start + desc.page_count * 4096;
            usable_start = usable_start.min(start);
            usable_end = usable_end.max(end);
        }
    }

    if usable_start < usable_end {
        Some((usable_start, usable_end))
    } else {
        None
    }
}

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
