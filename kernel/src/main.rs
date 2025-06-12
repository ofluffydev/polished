#![no_std]
#![no_main]

extern crate alloc;

use polished_bootloader::BootInfo;
use polished_files::ustar::tar_lookup;
use polished_interrupts::init_idt;
use polished_memory as _; // Import the memory module for memset, memcpy, etc.
use polished_panic_handler as _; // Import the panic handler.

use alloc::format;
use core::arch::{asm, naked_asm};
use linked_list_allocator::LockedHeap;
use polished_graphics::drawing::framebuffer_x_demo;
use polished_graphics::framebuffer::FramebufferInfo;
use polished_ps2::ps2_init;
use polished_serial_logging::{info, warn};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn naked_start() {
    // Set up the stack pointer to the top of the stack
    naked_asm!(
        "cli",
        "lea rsp, STACK_TOP",
        "call kernel_entry",
        "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

fn init_allocator() {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

fn log_framebuffer_info(fb: &FramebufferInfo) {
    let msg = format!(
        "FramebufferInfo: address=0x{:x}, size={}, {}x{}, stride={}, format=RAW",
        fb.address, fb.size, fb.width, fb.height, fb.stride
    );
    info(&msg);
}

fn clear_framebuffer(fb: &FramebufferInfo) {
    let buffer = unsafe { core::slice::from_raw_parts_mut(fb.address as *mut u8, fb.size) };
    for byte in buffer.iter_mut() {
        *byte = 0; // Fill with black
    }
    info("Framebuffer buffer filled with black");
    // Manually copy all fields, using a match for the format enum
    let mut fb_mut = FramebufferInfo {
        address: fb.address,
        size: fb.size,
        width: fb.width,
        height: fb.height,
        stride: fb.stride,
        format: match fb.format {
            polished_graphics::framebuffer::FramebufferFormat::Rgb => polished_graphics::framebuffer::FramebufferFormat::Rgb,
            polished_graphics::framebuffer::FramebufferFormat::Bgr => polished_graphics::framebuffer::FramebufferFormat::Bgr,
            polished_graphics::framebuffer::FramebufferFormat::Bitmask => polished_graphics::framebuffer::FramebufferFormat::Bitmask,
            polished_graphics::framebuffer::FramebufferFormat::BltOnly => polished_graphics::framebuffer::FramebufferFormat::BltOnly,
        },
    };
    framebuffer_x_demo(&mut fb_mut);
}

fn init_interrupts() {
    info("Loading IDT...");
    init_idt();
    info("IDT loaded");
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    init_allocator();
    let boot_info = unsafe { &*boot_info_ptr };
    let fb = if boot_info.framebuffer_bpp == 0 {
        warn("BootInfo.framebuffer_bpp is zero! Defaulting stride to 0 to avoid division by zero.");
        FramebufferInfo {
            address: boot_info.framebuffer_addr,
            size: (boot_info.framebuffer_pitch as usize) * (boot_info.framebuffer_height as usize),
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: 0,
            format: polished_graphics::framebuffer::FramebufferFormat::Rgb, // TODO: Use correct format if needed
        }
    } else {
        FramebufferInfo {
            address: boot_info.framebuffer_addr,
            size: (boot_info.framebuffer_pitch as usize) * (boot_info.framebuffer_height as usize),
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: boot_info.framebuffer_pitch as usize / (boot_info.framebuffer_bpp as usize / 8),
            format: polished_graphics::framebuffer::FramebufferFormat::Rgb, // TODO: Use correct format if needed
        }
    };
    info("Hello from the kernel!");
    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");
    init_interrupts();
    ps2_init();
    log_framebuffer_info(&fb);
    clear_framebuffer(&fb);
    x86_64::instructions::interrupts::enable();
    // Only disable the PIC after confirming interrupts work, or comment out for now
    // info("Disabling legacy PIC...");
    // disable_pic();
    // info("Legacy PIC disabled");
    // simulate_divide_by_zero();

    let ustar_archive = include_bytes!("../../archive.tar");

    // Example usage of the ustar module
    let file = match tar_lookup(ustar_archive, "ustar-files/mymessage.txt") {
        Some(file) => file,
        None => {
            warn("File not found in archive: mytext.txt");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: mytext.txt, size: {}", file.1));
    let file = match tar_lookup(ustar_archive, "ustar-files/hello_world.rs") {
        Some(file) => file,
        None => {
            warn("File not found in archive: hello_world.rs");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: hello_world.rs, size: {}", file.1));

    // Print the contents of the file
    let file_contents = core::str::from_utf8(file.0).expect("Invalid UTF-8 in file");
    info(&format!("File contents:\n{file_contents}"));

    // Loop forever to keep the kernel running
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    loop {
        unsafe { asm!("pause; hlt") }; // Use PAUSE before HLT for better power efficiency
    }

    // panic!("Kernel halted");
}
