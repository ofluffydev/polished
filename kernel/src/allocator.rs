use linked_list_allocator::LockedHeap;
use polished_bootloader::MEM_OFFSET;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_allocator() {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        ALLOCATOR.lock().init((heap_start + MEM_OFFSET.as_u64() as usize) as *mut u8, heap_size);
    }
}
