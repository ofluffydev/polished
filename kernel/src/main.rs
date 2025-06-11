#![no_std]
#![no_main]

extern crate alloc;

use polished_interrupts::init_idt;
use polished_memory as _;
use polished_panic_handler as _; // Import the panic handler // Import the memory module for memset, memcpy, etc.

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
    info("Loading IDT...");
    init_idt();
    info("IDT loaded");
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_entry(fb_info_ptr: *const FramebufferInfo) -> ! {
    init_allocator();
    info("Hello from the kernel!");
    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");
    init_interrupts();
    ps2_init();
    log_framebuffer_info(fb_info_ptr);
    clear_framebuffer(fb_info_ptr);
    x86_64::instructions::interrupts::enable();
    // Only disable the PIC after confirming interrupts work, or comment out for now
    // info("Disabling legacy PIC...");
    // disable_pic();
    // info("Legacy PIC disabled");
    // simulate_divide_by_zero();

    // Loop forever to keep the kernel running
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    loop {
        unsafe { asm!("hlt") }; // Halt the CPU until the next interrupt
    }

    // panic!("Kernel halted");
}
