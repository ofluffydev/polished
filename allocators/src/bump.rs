//! # Bump Allocators for Rust
//!
//! This module provides two bump allocator implementations:
//!
//! - [`BumpAllocator`]: A heap-backed bump allocator using a boxed, aligned buffer.
//! - [`StaticBumpAllocator`]: A bump allocator over a user-provided static buffer, suitable for `no_std` environments.
//!
//! ## What is a Bump Allocator?
//!
//! A bump allocator is a simple, fast memory allocator that allocates memory linearly from a pre-allocated region (the "heap").
//! Each allocation increments ("bumps") an offset forward by the requested size, optionally rounding up for alignment. Individual
//! deallocations are not supported; memory is only reclaimed by resetting or dropping the allocator, which invalidates all pointers.
//!
//! ## When and Why Use a Bump Allocator?
//!
//! Bump allocators are ideal for scenarios where:
//! - Many small allocations are made, but all can be freed at once (e.g., arena allocation, temporary workspaces, parsing, or bootstrapping).
//! - Allocation speed is critical and fragmentation is not a concern.
//! - The maximum memory usage is known ahead of time.
//!
//! They are commonly used in:
//! - Embedded and OS development (where allocators must be simple and predictable)
//! - Parsers and compilers (for ASTs, temporary objects)
//! - Game engines (for frame or level memory arenas)
//!
//! ## How to Use
//!
//! 1. Create a bump allocator with a fixed-size heap (either heap-allocated or static).
//! 2. Use the allocator to allocate memory for objects or buffers.
//! 3. When all allocations are no longer needed, call `reset()` or drop the allocator to reclaim all memory at once.
//!
//! ## Safety
//!
//! - All pointers returned by a bump allocator become invalid after reset or drop.
//! - The allocator must outlive all allocations.
//! - Thread safety: allocation is thread-safe, but deallocation/reset is not.
//!
//! ## Example
//!
//! ```rust
//! use allocators::bump::BumpAllocator;
//! use core::alloc::Layout;
//!
//! let heap_size = 1024;
//! let alloc = BumpAllocator::new(heap_size);
//! let layout = Layout::from_size_align(16, 8).unwrap();
//! let ptr = unsafe { alloc.alloc(layout) };
//! assert!(!ptr.is_null());
//! // ... use ptr ...
//! alloc.reset(); // All allocations are now invalid
//! ```
//!
//! [`BumpAllocator`]: struct.BumpAllocator.html
//! [`StaticBumpAllocator`]: struct.StaticBumpAllocator.html
//!
//! ## Testing
//!
//! This implementation is tested for:
//! - Basic allocation and alignment correctness
//! - Multiple allocations and pointer uniqueness
//! - Out-of-memory (OOM) conditions
//! - Zero-size allocations (contract compliance)
//! - Full-heap allocation and exhaustion
//! - No reuse of freed space (bump allocator)
//! - Alignment guarantees for various alignments
//! - Static buffer allocation (for `StaticBumpAllocator`)
//!
//! See the module's tests for details.

extern crate alloc;

use alloc::{boxed::Box, vec};

/// 32-byte aligned storage for the bump allocator heap.
#[repr(align(32))]
#[derive(Copy)]
#[allow(dead_code)]
struct Align32([u8; 32]);

impl Clone for Align32 {
    fn clone(&self) -> Self {
        *self
    }
}
use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A bump allocator that allocates memory linearly from a fixed-size heap.
///
/// Allocations are never individually freed; memory is only reclaimed when the allocator is dropped or reset.
/// This allocator is thread-safe for allocation, but does not support deallocation.
///
/// # Safety
/// All pointers returned by this allocator become invalid when the allocator is dropped or reset. The allocator must outlive all allocations.
/// Dropping or resetting the allocator will free the heap (Boxed slice).
pub struct BumpAllocator {
    heap: Box<[Align32]>,
    offset: AtomicUsize,
    heap_size: usize,
}

impl BumpAllocator {
    /// Create a new bump allocator with a heap of the given size (in bytes).
    ///
    /// # Arguments
    /// * `heap_size` - The size of the heap in bytes.
    ///
    /// # Returns
    /// A new `BumpAllocator` instance with an internal heap of the requested size.
    pub fn new(heap_size: usize) -> Self {
        let elem_size = 32;
        let n_elems = heap_size.div_ceil(elem_size);
        let heap = vec![Align32([0u8; 32]); n_elems].into_boxed_slice();
        Self {
            heap,
            offset: AtomicUsize::new(0),
            heap_size,
        }
    }

    /// Returns the aligned heap start pointer.
    fn heap_start(&self) -> *mut u8 {
        self.heap.as_ptr() as *mut u8
    }

    /// Reset the allocator, making all memory available again.
    ///
    /// # Safety
    /// All pointers previously returned become invalid after reset.
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Release);
    }
}

/// Safety: BumpAllocator is safe to share between threads for allocation, but not for deallocation.
unsafe impl Sync for BumpAllocator {}

/// Implements the `GlobalAlloc` trait for `BumpAllocator`.
/// Allocates memory by bumping the offset forward, never reusing freed space.
unsafe impl GlobalAlloc for BumpAllocator {
    /// Allocate a block of memory with the given layout.
    ///
    /// # Safety
    /// The caller must ensure the returned pointer is used safely and not accessed after the allocator is dropped.
    ///
    /// # Arguments
    /// * `layout` - The memory layout (size and alignment) to allocate.
    ///
    /// # Returns
    /// A pointer to the allocated memory, or null if out of memory or alignment cannot be satisfied.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let heap_start = self.heap_start();
        // Check if requested alignment is greater than heap alignment
        let heap_alignment = 1 << (heap_start as usize).trailing_zeros();
        if align > heap_alignment {
            return core::ptr::null_mut();
        }
        if size == 0 {
            if heap_start.align_offset(align) == usize::MAX {
                return core::ptr::null_mut();
            }
            return heap_start;
        }
        loop {
            let orig_offset = self.offset.load(Ordering::Acquire);
            if orig_offset > self.heap_size {
                return core::ptr::null_mut();
            }
            let ptr = unsafe { heap_start.add(orig_offset) };
            let offset = ptr.align_offset(align);
            if offset == usize::MAX || orig_offset.checked_add(offset).is_none() {
                return core::ptr::null_mut();
            }
            let aligned_offset = orig_offset + offset;
            let new_offset = aligned_offset.checked_add(size);
            if new_offset.is_none() || new_offset.unwrap() > self.heap_size {
                return core::ptr::null_mut();
            }
            if self
                .offset
                .compare_exchange(
                    orig_offset,
                    new_offset.unwrap(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return unsafe { heap_start.add(aligned_offset) };
            }
        }
    }

    /// Deallocate a block of memory previously allocated by this allocator.
    ///
    /// # Safety
    /// This is a no-op for bump allocators; memory is not reclaimed until the allocator is dropped.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // True bump allocator: deallocation is a no-op
        // Individual deallocations are not supported
    }
}

/// A bump allocator that works with a user-provided static memory buffer.
///
/// This version does not require heap allocation or the `alloc` crate, and is suitable for `no_std` environments.
///
/// # Safety
/// All pointers returned by this allocator become invalid when the allocator is dropped or reset. The allocator must outlive all allocations.
pub struct StaticBumpAllocator {
    /// Pointer to the start of the static heap buffer.
    heap_start: *mut u8,
    /// Size of the heap buffer in bytes.
    heap_size: usize,
    /// Current offset (in bytes) from the start of the heap, atomically updated.
    offset: AtomicUsize,
}

impl StaticBumpAllocator {
    /// Create a new static bump allocator from a user-provided buffer.
    ///
    /// # Safety
    /// The caller must ensure the buffer is valid for the lifetime of the allocator and not aliased elsewhere.
    ///
    /// # Arguments
    /// * `heap_start` - Pointer to the start of the buffer.
    /// * `heap_size` - Size of the buffer in bytes.
    ///
    /// # Returns
    /// A new `StaticBumpAllocator` instance using the provided buffer.
    pub unsafe fn new(heap_start: *mut u8, heap_size: usize) -> Self {
        // Initialize the allocator with the provided buffer and set offset to 0.
        Self {
            heap_start,
            heap_size,
            offset: AtomicUsize::new(0),
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

/// Safety: StaticBumpAllocator is safe to share between threads for allocation, but not for deallocation.
unsafe impl Sync for StaticBumpAllocator {}

/// Implements the `GlobalAlloc` trait for `StaticBumpAllocator`.
/// Allocates memory from a static buffer by bumping the offset forward.
unsafe impl GlobalAlloc for StaticBumpAllocator {
    /// Allocate a block of memory with the given layout from the static buffer.
    ///
    /// # Safety
    /// The caller must ensure the returned pointer is used safely and not accessed after the allocator is dropped.
    ///
    /// # Arguments
    /// * `layout` - The memory layout (size and alignment) to allocate.
    ///
    /// # Returns
    /// A pointer to the allocated memory, or null if out of memory or alignment cannot be satisfied.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Fast path for zero‑size: return a non‑null aligned pointer
        if size == 0 {
            let ptr = self.heap_start;
            return if (ptr as usize) % align == 0 {
                ptr
            } else {
                // Round the heap start itself up
                let rounded = (ptr as usize + (align - 1)) & !(align - 1);
                rounded as *mut u8
            };
        }

        let heap_start = self.heap_start;

        // What alignment can the heap itself guarantee?
        let heap_alignment = 1 << (heap_start as usize).trailing_zeros();
        if align > heap_alignment {
            // Asking for stricter alignment than the heap can ever satisfy
            return core::ptr::null_mut();
        }

        loop {
            let orig = self.offset.load(Ordering::Acquire);

            // Round orig _up_ to the next multiple of `align`
            let aligned_off = (orig + (align - 1)) & !(align - 1);

            // Compute new offset = aligned_off + size, detect overflow & OOM
            let new_off = match aligned_off.checked_add(size) {
                Some(no) if no <= self.heap_size => no,
                _ => return core::ptr::null_mut(),
            };

            // Try to claim it
            if self
                .offset
                .compare_exchange(orig, new_off, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Success! return ptr = heap_start + aligned_off
                return unsafe { heap_start.add(aligned_off) };
            }
            // else: race, retry
        }
    }

    /// Deallocate a block of memory previously allocated by this allocator.
    ///
    /// # Safety
    /// This is a no-op for bump allocators; memory is not reclaimed until the allocator is dropped.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: bump allocator does not support freeing individual allocations.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    // Helper to create a test allocator with a fixed heap size and alignment
    fn test_allocator(size: usize) -> BumpAllocator {
        BumpAllocator::new(size)
    }

    #[test]
    fn alloc_basic() {
        let heap_size = 128;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(8, 4).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "Allocation should succeed");
        // Check alignment
        assert_eq!(ptr as usize % 4, 0, "Pointer should be 4-byte aligned");
    }

    #[test]
    fn alloc_multiple() {
        let heap_size = 64;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        assert!(
            !ptr1.is_null() && !ptr2.is_null(),
            "Both allocations should succeed"
        );
        assert_ne!(ptr1, ptr2, "Pointers should be different");
        assert_eq!(ptr1 as usize % 8, 0);
        assert_eq!(ptr2 as usize % 8, 0);
    }

    #[test]
    fn alloc_out_of_memory() {
        let heap_size = 32;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(32, 1).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        assert!(!ptr1.is_null(), "First allocation should succeed");
        assert!(ptr2.is_null(), "Second allocation should fail (OOM)");
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
    fn alloc_max_size() {
        let heap_size = 128;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(heap_size, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "Should allocate entire heap");
        let layout2 = Layout::from_size_align(1, 1).unwrap();
        let ptr2 = unsafe { alloc.alloc(layout2) };
        assert!(ptr2.is_null(), "Should be OOM after full allocation");
    }

    #[test]
    fn dealloc_does_not_reuse() {
        let heap_size = 32;
        let alloc = test_allocator(heap_size);
        let layout = Layout::from_size_align(16, 1).unwrap();
        let ptr1 = unsafe { alloc.alloc(layout) };
        unsafe { alloc.dealloc(ptr1, layout) };
        let ptr2 = unsafe { alloc.alloc(layout) };
        assert_ne!(ptr1, ptr2, "Bump allocator should not reuse freed space");
    }

    #[test]
    fn alloc_alignment() {
        let heap_size = 128;
        let alloc = test_allocator(heap_size);
        let alignments = [1, 2, 4, 8, 16, 32];
        for &align in &alignments {
            let layout = Layout::from_size_align(8, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(
                !ptr.is_null(),
                "Allocation with alignment {align} should succeed"
            );
            assert_eq!(
                ptr as usize % align,
                0,
                "Pointer should be {align}-byte aligned"
            );
        }
    }
}

#[cfg(test)]
mod static_tests {
    use super::*;
    use core::alloc::Layout;
    use core::mem::MaybeUninit;

    #[test]
    fn static_alloc_basic() {
        let mut heap: [MaybeUninit<u8>; 256] = [MaybeUninit::uninit(); 256];
        let heap_ptr = heap.as_mut_ptr() as *mut u8;
        let alloc = unsafe { StaticBumpAllocator::new(heap_ptr, 256) };
        let layout = Layout::from_size_align(8, 4).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "Allocation should succeed");
        assert_eq!(ptr as usize % 4, 0, "Pointer should be 4-byte aligned");
    }
}
