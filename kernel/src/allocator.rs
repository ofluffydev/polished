use limine::response::MemoryMapResponse;
use linked_list_allocator::LockedHeap;
use polished_allocators::frame::{find_usable_region, get_frame_allocator};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
};

use crate::write_str;

// Place heap 10 MiB after the kernel (e.g., at 0xffffffff80a00000)
pub const HEAP_START: u64 = 0xffffffff80a00000; // 10 MiB after kernel base
pub const HEAP_SIZE: u64 = 0x0100_0000; // 16 MB
pub const HEAP_SIZE_USIZE: usize = 0x0100_0000; // 16 MB

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn get_hhdm_offset() -> u64 {
    crate::HHDM_REQUEST.get_response().unwrap().offset()
}

pub fn init_allocator() {
    let heap_start = HEAP_START as *mut u8;
    let heap_size = HEAP_SIZE_USIZE;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

/// # Safety
/// The caller must guarantee that the complete physical memory is mapped
/// at the passed `physical_memory_offset`.
unsafe fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

pub fn map_heap_region(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    heap_virt_start: VirtAddr,
    heap_phys_start: PhysAddr,
    heap_size: u64,
) {
    write_str("Mapping heap region...");
    let heap_end = heap_virt_start + heap_size - 1;
    let start_page = Page::<Size4KiB>::containing_address(heap_virt_start);
    let end_page = Page::<Size4KiB>::containing_address(heap_end);
    let page_range = Page::range_inclusive(start_page, end_page);

    let mut phys_addr = heap_phys_start;

    write_str("Mapping pages...");
    for page in page_range {
        let frame = PhysFrame::containing_address(phys_addr);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // Safety: We assume the region is not yet mapped and the frame is valid.
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("map_to failed")
                .flush();
        }
        phys_addr += Size4KiB::SIZE;
    }
    write_str("Heap region mapped successfully");
}

pub fn setup_heap_space(memory_map: &MemoryMapResponse) {
    write_str("Setting up heap space...");
    let (heap_phys_start, _heap_region_len) =
        find_usable_region(memory_map, HEAP_SIZE_USIZE).expect("No suitable memory region found");
    let heap_virt_start = VirtAddr::new(HEAP_START);
    let heap_phys_start = PhysAddr::new(heap_phys_start);

    write_str("Initializing memory mapper...");
    let mut mapper = unsafe {
        let physical_memory_offset = VirtAddr::new(get_hhdm_offset());
        init_mapper(physical_memory_offset)
    };
    write_str("Memory mapper initialized");

    write_str("Getting frame allocator...");
    let mut frame_allocator = get_frame_allocator(memory_map, HEAP_SIZE_USIZE);
    write_str("Frame allocator obtained");

    // Only map the heap size, not the whole region
    write_str("Mapping heap region to virtual memory...");
    map_heap_region(
        &mut mapper,
        &mut frame_allocator,
        heap_virt_start,
        heap_phys_start,
        HEAP_SIZE,
    );
}
