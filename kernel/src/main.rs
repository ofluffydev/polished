#![no_std]
#![no_main]

#[cfg(not(test))]
use core::panic::PanicInfo;

use buddy_system_allocator::LockedHeap;

pub mod memory;

extern crate alloc;
use alloc::format;
use graphics::drawing::framebuffer_x_demo;
use graphics::framebuffer::FramebufferInfo;

// False positive error shows here for rust analyzer, ignore it.
// Sometimes saving the file will fix it.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<33> = LockedHeap::<33>::empty();

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start(fb_info_ptr: *const FramebufferInfo) -> ! {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    }

    serial_logging::info("Hello from the kernel!");

    // Log framebuffer info if pointer is not null
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &*fb_info_ptr };
        let msg = format!(
            "FramebufferInfo: address=0x{:x}, size={}, {}x{}, stride={}, format={:?}",
            fb.address, fb.size, fb.width, fb.height, fb.stride, fb.format
        );
        serial_logging::info(&msg);
    } else {
        serial_logging::info("FramebufferInfo pointer is null");
    }

    // Fill the buffer with black
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &mut *(fb_info_ptr as *mut FramebufferInfo) };
        let buffer = unsafe { core::slice::from_raw_parts_mut(fb.address as *mut u8, fb.size) };
        for byte in buffer.iter_mut() {
            *byte = 0; // Fill with black
        }
        serial_logging::info("Framebuffer buffer filled with black");

        framebuffer_x_demo(fb);
    } else {
        serial_logging::warn("FramebufferInfo pointer is null, cannot fill buffer");
    }

    panic!("Kernel halted");
}
