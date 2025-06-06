#![no_std]
#![no_main]

#[cfg(not(test))]
use core::panic::PanicInfo;

use buddy_system_allocator::LockedHeap;

pub mod memory;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<33> = LockedHeap::<33>::empty();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    }

    serial_logging::info("Hello from the kernel!");

    panic!("Kernel halted");
}
