#![no_std]
#![no_main]

use core::arch::{asm, naked_asm};
#[cfg(not(test))]
use core::panic::PanicInfo;

use alloc::string::ToString;

pub mod gdt;
pub mod memory;

extern crate alloc;
use alloc::format;
use graphics::drawing::framebuffer_x_demo;
use graphics::framebuffer::FramebufferInfo;
use serial_logging::{info, serial_write_str, warn};

pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    serial_write_str("=== PANIC ===\n");

    #[cfg(not(debug_assertions))]
    serial_write_str("[WARNING] This is a release build, panic information may be limited.\n");

    if let Some(location) = info.location() {
        serial_write_str("Location: ");
        serial_write_str(location.file());
        serial_write_str(":");
        serial_write_str(location.line().to_string().as_str());
        serial_write_str(":");
        serial_write_str(location.column().to_string().as_str());
        serial_write_str("\n");
    } else {
        serial_write_str("Location: <unknown>\n");
    }

    if let Some(message) = info.message().as_str() {
        serial_write_str("Message: ");
        serial_write_str(message);
        serial_write_str("\n");
    } else {
        serial_write_str("Message: <none>\n");
    }

    serial_write_str("=============\n");
    serial_write_str("\n\n");
}

// False positive error shows here for rust analyzer, ignore it.
// Sometimes saving the file will fix it.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_logging::error("Kernel panic occurred!");
    print_panic_info_serial(info);
    loop {
        // Halt the CPU to prevent further execution
        unsafe { asm!("cli", "hlt") };
    }
}

use linked_list_allocator::LockedHeap;

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

/// Disable the PIC
pub fn disable_pic() {
    info("Disabling PIC...");
    unsafe {
        // Mask all interrupts on both PICs
        asm!(
            "mov al, 0xFF",
            "out 0xA1, al", // slave PIC
            "out 0x21, al", // master PIC
            options(nostack, nomem, preserves_flags)
        );
    }
}

fn init_allocator() {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

fn log_framebuffer_info(fb_info_ptr: *const FramebufferInfo) {
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &*fb_info_ptr };
        let msg = format!(
            "FramebufferInfo: address=0x{:x}, size={}, {}x{}, stride={}, format={:?}",
            fb.address, fb.size, fb.width, fb.height, fb.stride, fb.format
        );
        info(&msg);
    } else {
        info("FramebufferInfo pointer is null");
    }
}

fn clear_framebuffer(fb_info_ptr: *const FramebufferInfo) {
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &mut *(fb_info_ptr as *mut FramebufferInfo) };
        let buffer = unsafe { core::slice::from_raw_parts_mut(fb.address as *mut u8, fb.size) };
        for byte in buffer.iter_mut() {
            *byte = 0; // Fill with black
        }
        info("Framebuffer buffer filled with black");
        framebuffer_x_demo(fb);
    } else {
        warn("FramebufferInfo pointer is null, cannot fill buffer");
    }
}

fn init_interrupts() {
    serial_logging::info("Loading IDT...");
    interrupts::init_idt();
    serial_logging::info("IDT loaded");
}

fn simulate_divide_by_zero() {
    #[allow(unconditional_panic)]
    let _ = 1 / 0; // This should trigger a divide by zero exception
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_entry(fb_info_ptr: *const FramebufferInfo) -> ! {
    init_allocator();
    info("Hello from the kernel!");
    info("Disabling legacy PIC...");
    disable_pic();
    info("Legacy PIC disabled");
    info("Initializing GDT...");
    gdt::init_gdt();
    info("GDT initialized");
    init_interrupts();
    log_framebuffer_info(fb_info_ptr);
    clear_framebuffer(fb_info_ptr);
    x86_64::instructions::interrupts::enable();
    simulate_divide_by_zero();
    panic!("Kernel halted");
}
