#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;

use limine::response::MemoryMapResponse;
use polished_memory as _;
use polished_panic_handler as _;

use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

use polished_memory as _;
use polished_panic_handler as _;
use polished_ps2::ps2_init;
use polished_serial_logging::info;

use crate::allocator::init_allocator;
use crate::interrupts::init_interrupts;

// Internal modules
mod allocator;
pub mod demos;
mod framebuffer_utils;
mod interrupts;

fn get_memory_map() -> &'static MemoryMapResponse {
    MEMORY_MAP_REQUEST.get_response().unwrap()
}

/// Early boot no-alloc function to write a string to the serial console.
pub fn write_str(s: &str) {
    unsafe {
        let prefix = "[write_str]: ";
        asm!(
            "rep outs dx, byte ptr [rsi]",
            in("dx") 0xe9,
            inout("rsi") prefix.as_ptr() => _,
            inout("rcx") prefix.len() => _,
            options(readonly, nostack)
        );
        asm!(
            "rep outs dx, byte ptr [rsi]",
            in("dx") 0xe9,
            inout("rsi") s.as_ptr() => _,
            inout("rcx") s.len() => _,
            options(readonly, nostack)
        );
        let newline = "\r\n";
        asm!(
            "rep outs dx, byte ptr [rsi]",
            in("dx") 0xe9,
            inout("rsi") newline.as_ptr() => _,
            inout("rcx") newline.len() => _,
            options(readonly, nostack)
        );
    }
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `boot_info_ptr` must be a valid pointer to a `BootInfo` structure, or null.
#[unsafe(export_name = "kmain")]
unsafe extern "C" fn kernel_entry() -> ! {
    write_str("Kernel entry point reached!");
    assert!(BASE_REVISION.is_supported());
    if let Some(_framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        // You can use the framebuffer here if you want, but leaving as placeholder per user request
    }
    write_str("Base revision is supported!");

    // Unlike previous custom bootloader, we must setup memory ourselves
    write_str("Getting memory map...");
    let memory_map = get_memory_map();
    write_str("Memory map obtained!");

    write_str("Setting up room for heap...");
    allocator::setup_heap_space(memory_map);
    write_str("Room for heap set up!");

    // Initialize heap allocator
    write_str("Initializing heap allocator...");
    init_allocator();
    write_str("Heap allocator initialized!");
    write_str("Kernel heap memory should be ready, switching to alloc logging now...");

    // If this crashes, something is wrong with the heap allocator setup
    info("Hello from the kernel!");

    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");

    // Set up interrupts and PS/2
    init_interrupts();
    ps2_init();

    // QEMU pci-testdev demo
    crate::demos::pci_testdev_demo();
    info("PCI test device demo completed");

    // Demo: extract and print files from ustar archive
    let ustar_archive = include_bytes!("../../archive.tar");
    crate::demos::demo_ustar_archive(ustar_archive);

    // Main kernel loop
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    hcf();
}

// #[unsafe(no_mangle)]
// unsafe extern "C" fn kmain() -> ! {
//     write_str("Reached kernel main!\r\n");
//     // All limine requests must also be referenced in a called function, otherwise they may be
//     // removed by the linker.
//     assert!(BASE_REVISION.is_supported());
//     write_str("Base revision is supported!\r\n");

//     write_str("Initializing heap allocator...\r\n");
//     allocator::init_allocator();
//     write_str("Allocator initialized!\r\n");

//     write_str("Initializing GDT...\r\n");
//     polished_gdt::init_gdt();
//     write_str("GDT initialized\r\n");

//     // Set up interrupts and PS/2
//     write_str("Initializing interrupts...\r\n");
//     interrupts::init_interrupts();
//     write_str("Interrupts initialized\r\n");
//     polished_ps2::ps2_init();
//     write_str("PS/2 initialized\r\n");

//     if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response()
//         && let Some(framebuffer) = framebuffer_response.framebuffers().next()
//     {
//         for i in 0..100_u64 {
//             write_str("drawing...");
//             // Calculate the pixel offset using the framebuffer information we obtained above.
//             // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
//             let pixel_offset = i * framebuffer.pitch() + i * 4;

//             // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
//             unsafe {
//                 framebuffer
//                     .addr()
//                     .add(pixel_offset as usize)
//                     .cast::<u32>()
//                     .write(0xFFFFFFFF)
//             };
//         }
//     }

//     hcf();
// }

/// Halt and Catch Fire (HCF) function.
fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
