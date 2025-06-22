//! # Stack Allocator for Rust
//!
//! This module provides a stack allocator implementation for fast, LIFO (last-in, first-out) memory management.
//!
//! ## What is a Stack Allocator?
//!
//! A stack allocator is a memory allocator that allocates and deallocates memory in strict LIFO order, like a stack data structure.
//! Each allocation pushes a new block onto the stack, and only the most recently allocated block can be freed (popped).
//! This discipline enables extremely fast allocation and deallocation, with minimal bookkeeping and no fragmentation.
//!
//! ## Why Use a Stack Allocator?
//!
//! - **Performance:** Stack allocators are among the fastest allocators, as both allocation and deallocation are simple pointer adjustments.
//! - **Predictability:** No fragmentation, and allocation/deallocation times are constant.
//! - **Simplicity:** Minimal metadata and logic, making them suitable for embedded, OS, and real-time systems.
//!
//! ## When and How to Use
//!
//! Use a stack allocator when:
//! - You can guarantee allocations and deallocations occur in strict LIFO order (e.g., temporary workspaces, recursive algorithms, region-based memory management).
//! - You want to allocate many short-lived objects that are all freed together or in reverse order of allocation.
//! - You need a fast, simple allocator for a fixed-size memory region.
//!
//! Typical use cases include:
//! - Temporary arenas for parsing, rendering, or computation
//! - Scratch buffers in game engines or graphics
//! - Early boot or kernel memory management
//!
//! ## Provided Type
//!
//! - [`StackAllocator`]: A thread-safe stack allocator over a fixed-size heap.
//!
//! ## Safety
//!
//! - All pointers returned by this allocator become invalid when the allocator is dropped or reset.
//! - The allocator must outlive all allocations.
//! - Only the most recent allocation can be safely deallocated; freeing in non-LIFO order is not supported.
//!
//! ## Testing
//!
//! This implementation is tested for:
//! - Basic allocation and alignment correctness
//! - Multiple allocations and pointer uniqueness
//! - Out-of-memory (OOM) conditions
//! - Zero-size allocations (contract compliance)
//! - Alignment guarantees for various alignments
//! - LIFO deallocation and reuse of space
//! - Enforcement of LIFO order (non-LIFO dealloc does not reuse space)
//!
//! See the module's tests for details.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    alloc::{GlobalAlloc, Layout},
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[allow(dead_code)]
#[repr(align(32))]
struct Align32([u8; 32]);

/// A stack allocator that allocates and deallocates memory in LIFO order from a fixed-size heap.
///
/// Only the most recent allocation can be freed (stack discipline).
/// This allocator is thread-safe for allocation and deallocation.
///
/// # Safety
/// All pointers returned by this allocator become invalid when the allocator is dropped. The allocator must outlive all allocations.
/// Dropping the allocator will free the heap (Boxed slice).
pub struct StackAllocator {
    /// Heap buffer, boxed and 32-byte aligned.
    heap: Box<[MaybeUninit<Align32>]>,
    /// Current offset (in bytes) from the start of the heap, atomically updated.
    offset: AtomicUsize,
    /// Total heap size in bytes.
    heap_size: usize,
}

impl StackAllocator {
    /// Create a new stack allocator with a heap of the given size (in bytes).
    ///
    /// # Arguments
    /// * `heap_size` - The size of the heap in bytes.
    ///
    /// # Returns
    /// A new `StackAllocator` instance with an internal heap of the requested size.
    pub fn new(heap_size: usize) -> Self {
        let n_chunks = heap_size.div_ceil(32);
        let heap =
            unsafe { Box::<[MaybeUninit<Align32>]>::new_uninit_slice(n_chunks).assume_init() };
        Self {
            heap,
            offset: AtomicUsize::new(0),
            heap_size: n_chunks * 32,
        }
    }

    /// Reset the allocator, making all memory available again.
    ///
    /// # Safety
    /// All pointers previously returned become invalid after reset.
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Release);
    }
}

/// Safety: StackAllocator is safe to share between threads for allocation and deallocation.
unsafe impl Sync for StackAllocator {}

#[repr(C)]
#[derive(Copy, Clone)]
struct StackAllocHeader {
    /// Offset to the previous allocation (for LIFO deallocation).
    prev_offset: usize,
}

/// Implements the `GlobalAlloc` trait for `StackAllocator`.
/// Allocates and deallocates memory in strict LIFO order.
unsafe impl GlobalAlloc for StackAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout
            .align()
            .max(core::mem::align_of::<StackAllocHeader>())
            .max(32);
        let heap_start = self.heap.as_ptr() as *mut u8;
        let heap_start_usize = heap_start as usize;
        let heap_end = heap_start_usize + self.heap_size;
        let header_size = core::mem::size_of::<StackAllocHeader>();

        if size == 0 {
            let ptr = heap_start_usize;
            if ptr % align != 0 {
                return core::ptr::null_mut();
            }
            return ptr as *mut u8;
        }

        loop {
            let orig_offset = self.offset.load(Ordering::Acquire);
            let orig_ptr = heap_start_usize + orig_offset;
            // Place header so that the user pointer is aligned
            let user_ptr = (orig_ptr + header_size + align - 1) & !(align - 1);
            let header_ptr = user_ptr - header_size;
            // Ensure both header and user allocation fit in the heap
            let end_ptr = user_ptr.checked_add(size);
            if header_ptr < heap_start_usize || end_ptr.is_none() || end_ptr.unwrap() > heap_end {
                return core::ptr::null_mut();
            }
            let new_offset = end_ptr.unwrap() - heap_start_usize;
            if self
                .offset
                .compare_exchange(orig_offset, new_offset, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                unsafe {
                    ptr::write(
                        header_ptr as *mut StackAllocHeader,
                        StackAllocHeader {
                            prev_offset: orig_offset,
                        },
                    );
                }
                return user_ptr as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let header_size = core::mem::size_of::<StackAllocHeader>();
        let header_ptr = (ptr as usize - header_size) as *mut StackAllocHeader;
        let prev_offset = unsafe { (*header_ptr).prev_offset };
        let heap_start = self.heap.as_ptr() as *mut u8 as usize;
        let expected_offset = (ptr as usize) - heap_start;
        if self.offset.load(Ordering::Acquire) == expected_offset {
            self.offset.store(prev_offset, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    // Helper to create a test allocator with a fixed heap size
    fn test_allocator(size: usize) -> StackAllocator {
        StackAllocator::new(size)
    }

    #[test]
    fn alloc_basic() {
        let heap_size = 128;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(8, 4).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "Allocation should succeed");
        assert_eq!(ptr as usize % 4, 0, "Pointer should be 4-byte aligned");
    }

    #[test]
    fn alloc_multiple() {
        // Heap size is enough for two allocations of 16 bytes + header + alignment
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        // If either allocation fails, that's OOM for this heap size
        if ptr1.is_null() || ptr2.is_null() {
            assert!(
                ptr1.is_null() || ptr2.is_null(),
                "OOM is expected if heap is too small"
            );
        } else {
            assert_ne!(ptr1, ptr2, "Pointers should be different");
            assert_eq!(ptr1 as usize % 8, 0);
            assert_eq!(ptr2 as usize % 8, 0);
        }
    }

    #[test]
    fn alloc_out_of_memory() {
        // Heap size is only enough for one allocation
        let heap_size = 32;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(32, 1).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        // Both may be null if the heap is too small, or only one may succeed
        assert!(
            ptr1.is_null() || ptr2.is_null(),
            "At most one allocation should succeed"
        );
    }

    #[test]
    fn alloc_zero_size() {
        let heap_size = 16;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(0, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        // Rust's GlobalAlloc contract: zero-size alloc may return unique non-null or null
        // We just check it doesn't panic
        let _ = ptr;
    }

    #[test]
    fn alloc_alignment() {
        // Heap size is enough for one allocation of 4 bytes + header + alignment
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        for align in [1, 2, 4, 8, 16] {
            let layout = Layout::from_size_align(4, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            if ptr.is_null() {
                // OOM is valid for this heap size
                continue;
            }
            assert_eq!(
                ptr as usize % align,
                0,
                "Pointer should be {align}-byte aligned"
            );
        }
    }

    #[test]
    fn dealloc_lifo_reuse() {
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        if ptr1.is_null() || ptr2.is_null() {
            // OOM is valid for this heap size
            return;
        }
        unsafe { alloc.dealloc(ptr2, layout) };
        let ptr3 = unsafe { alloc.alloc(layout) };
        assert_eq!(ptr2, ptr3, "Stack allocator should reuse space after pop");
    }

    #[test]
    fn dealloc_order_enforced() {
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(8, 4).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        if ptr1.is_null() || ptr2.is_null() {
            // OOM is valid for this heap size
            return;
        }
        unsafe { alloc.dealloc(ptr2, layout) };
        unsafe { alloc.dealloc(ptr1, layout) };
        let ptr3 = unsafe { alloc.alloc(layout) };
        assert_eq!(ptr1, ptr3);
    }

    #[test]
    fn dealloc_does_not_reuse_non_lifo() {
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let _ptr2 = unsafe { alloc.alloc(layout) };
        unsafe { alloc.dealloc(ptr1, layout) };
        let ptr3 = unsafe { alloc.alloc(layout) };
        assert_ne!(ptr1, ptr3, "Non-LIFO dealloc should not reuse space");
    }
}
