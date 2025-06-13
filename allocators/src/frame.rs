//! # Physical Frame Allocators for Rust
//!
//! This module provides abstractions and implementations for physical memory frame allocation.
//! Frame allocators are a core component of operating system kernels and low-level memory managers.
//!
//! ## What is a Frame Allocator?
//!
//! A frame allocator manages physical memory in fixed-size blocks called "frames" (typically 4 KiB each).
//! It is responsible for handing out unused frames for use by the kernel, page tables, or user processes,
//! and for reclaiming frames when they are no longer needed.
//!
//! ## Why Use Frame Allocators?
//!
//! - **Paging and Virtual Memory:** Frame allocators are essential for mapping virtual memory to physical memory.
//! - **OS Kernels:** Any kernel that manages its own memory (paging, heap, stacks) needs a way to allocate and free physical frames.
//! - **Predictability:** By using fixed-size frames, fragmentation is minimized and allocation is fast and simple.
//!
//! ## When and How to Use
//!
//! - Use a frame allocator when you need to allocate or free physical memory for page tables, kernel heaps, or user processes.
//! - Choose a bump allocator for simple, one-shot allocation (e.g., early boot, when you never free frames).
//! - Choose a free-list allocator when you need to support freeing and reusing frames (e.g., after boot, for dynamic memory management).
//!
//! ## Provided Types
//!
//! - [`PhysFrame`]: Represents a single physical frame of memory.
//! - [`FrameAllocator`]: Trait for frame allocators (allocate and deallocate frames).
//! - [`BumpFrameAllocator`]: Simple bump allocator for frames (fast, no reuse).
//! - [`FreeListFrameAllocator`]: Free-list allocator for frames (supports reuse).
//!
//! ## Safety
//!
//! - The caller must ensure that the memory region given to an allocator is valid and not used elsewhere.
//! - Allocators do not check for aliasing or overlapping regions.
//! - All frame addresses are aligned to `FRAME_SIZE`.
//!
//! ## Testing
//!
//! The implementations here are tested for:
//! - Correct alignment and address calculation for frames
//! - Exhaustion and out-of-memory conditions
//! - No reuse in bump allocator, correct reuse in free-list allocator
//! - Double-free handling in free-list allocator
//! - Correct allocation order (LIFO) in free-list allocator
//! - Handling of zero-sized and unaligned regions
//!
//! See the module's tests for details.

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::vec::Vec;

/// Size of a physical frame in bytes.
pub const FRAME_SIZE: usize = 4096;

/// Represents a physical frame of memory.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct PhysFrame {
    pub start_address: usize,
}

impl PhysFrame {
    /// Returns the frame containing the given address.
    ///
    /// The returned frame's start address is aligned down to the nearest frame boundary.
    pub const fn containing_address(addr: usize) -> Self {
        PhysFrame {
            start_address: addr & !(FRAME_SIZE - 1),
        }
    }

    /// Returns the start address of this frame.
    pub const fn start_address(&self) -> usize {
        self.start_address
    }
}

/// Trait for a physical frame allocator.
///
/// Provides methods to allocate and deallocate physical frames.
pub trait FrameAllocator {
    /// Allocates a frame. Returns `None` if out of memory.
    fn allocate_frame(&mut self) -> Option<PhysFrame>;

    /// Frees a frame, making it available for reuse.
    fn deallocate_frame(&mut self, frame: PhysFrame);
}

/// A simple bump allocator for physical memory frames.
///
/// Allocates frames linearly from a region, never reusing freed frames.
/// Fast and simple, but cannot reclaim memory until reset or dropped.
/// Useful for early boot or one-shot allocation scenarios.
pub struct BumpFrameAllocator {
    start: usize,
    end: usize,
    next: AtomicUsize,
}

impl BumpFrameAllocator {
    /// Creates a new bump frame allocator for the given region.
    ///
    /// # Safety
    /// Caller must ensure the region is valid and not used elsewhere.
    pub const unsafe fn new(start: usize, end: usize) -> Self {
        BumpFrameAllocator {
            start,
            end,
            next: AtomicUsize::new(start),
        }
    }
}

impl FrameAllocator for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let current = self.next.load(Ordering::Relaxed);
        let aligned = (current + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);

        if aligned + FRAME_SIZE <= self.end {
            self.next.store(aligned + FRAME_SIZE, Ordering::Relaxed);
            Some(PhysFrame {
                start_address: aligned,
            })
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, _frame: PhysFrame) {
        // No-op: bump allocator cannot reuse frames without a free list.
    }
}

impl fmt::Debug for BumpFrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BumpFrameAllocator")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("next", &self.next.load(Ordering::Relaxed))
            .finish()
    }
}

/// A free list allocator for physical memory frames.
///
/// Maintains a list of free frames and supports allocation and deallocation.
/// Suitable for dynamic memory management after boot.
pub struct FreeListFrameAllocator {
    free_list: Vec<PhysFrame>,
}

impl FreeListFrameAllocator {
    /// Creates a new free list frame allocator for the given region.
    ///
    /// # Safety
    /// The given range must be frame-aligned and not overlap with used memory.
    pub unsafe fn new(start: usize, end: usize) -> Self {
        let mut free_list = Vec::new();
        let mut addr = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        while addr + FRAME_SIZE <= end {
            free_list.push(PhysFrame {
                start_address: addr,
            });
            addr += FRAME_SIZE;
        }
        FreeListFrameAllocator { free_list }
    }
}

impl FrameAllocator for FreeListFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.free_list.pop()
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.free_list.push(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physframe_alignment() {
        let addr = 0x12345;
        let frame = PhysFrame::containing_address(addr);
        assert_eq!(frame.start_address % FRAME_SIZE, 0);
        assert!(addr >= frame.start_address);
        assert!(addr < frame.start_address + FRAME_SIZE);
    }

    #[test]
    fn physframe_zero_address() {
        let frame = PhysFrame::containing_address(0);
        assert_eq!(frame.start_address, 0);
    }

    #[test]
    fn physframe_unaligned_address() {
        let addr = FRAME_SIZE * 5 + 123;
        let frame = PhysFrame::containing_address(addr);
        assert_eq!(frame.start_address, FRAME_SIZE * 5);
        assert!(addr >= frame.start_address);
        assert!(addr < frame.start_address + FRAME_SIZE);
    }

    #[test]
    fn bump_allocator_basic() {
        let start = 0x10000;
        let end = start + 3 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame();
        let f2 = alloc.allocate_frame();
        let f3 = alloc.allocate_frame();
        assert!(f1.is_some() && f2.is_some() && f3.is_some());
        assert_ne!(f1, f2);
        assert_ne!(f2, f3);
        assert_ne!(f1, f3);
        // Should be exhausted now
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn bump_allocator_no_reuse() {
        let start = 0x20000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(f1);
        let f2 = alloc.allocate_frame().unwrap();
        assert_ne!(f1, f2, "Bump allocator must not reuse frames");
    }

    #[test]
    fn bump_allocator_zero_region() {
        let mut alloc = unsafe { BumpFrameAllocator::new(0, 0) };
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn bump_allocator_unaligned_start() {
        let start = 0x12345;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame();
        assert!(f1.is_some());
        assert_eq!(f1.unwrap().start_address % FRAME_SIZE, 0);
    }

    #[test]
    fn bump_allocator_exhaustion() {
        let start = 0x80000;
        let end = start + FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame();
        let f2 = alloc.allocate_frame();
        assert!(f1.is_some());
        assert!(f2.is_none());
    }

    #[test]
    fn freelist_allocator_basic() {
        let start = 0x30000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame();
        let f2 = alloc.allocate_frame();
        assert!(f1.is_some() && f2.is_some());
        assert_ne!(f1, f2);
        // Should be exhausted now
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn freelist_allocator_reuse() {
        let start = 0x40000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(f1);
        let f2 = alloc.allocate_frame().unwrap();
        assert_eq!(f1, f2, "Freelist should reuse deallocated frames");
    }

    #[test]
    fn freelist_allocator_double_free() {
        let start = 0x50000;
        let end = start + FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(f);
        alloc.deallocate_frame(f); // double free
        // Should be able to allocate twice, but not a third time
        assert!(alloc.allocate_frame().is_some());
        assert!(alloc.allocate_frame().is_some());
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn freelist_allocator_alignment() {
        let start = 0x12345;
        let end = start + 3 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let aligned_start = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        let n_frames = if end > aligned_start {
            (end - aligned_start) / FRAME_SIZE
        } else {
            0
        };
        for _ in 0..n_frames {
            let f = alloc.allocate_frame().unwrap();
            assert_eq!(f.start_address % FRAME_SIZE, 0);
        }
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn freelist_allocator_zero_region() {
        let mut alloc = unsafe { FreeListFrameAllocator::new(0, 0) };
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn freelist_allocator_unaligned_start() {
        let start = 0x12345;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame();
        assert!(f1.is_some());
        assert_eq!(f1.unwrap().start_address % FRAME_SIZE, 0);
    }

    #[test]
    fn freelist_allocator_stress_many_frames() {
        let start = 0x100000;
        let n = 100;
        let end = start + n * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let mut frames = Vec::new();
        for _ in 0..n {
            let f = alloc.allocate_frame();
            assert!(f.is_some());
            frames.push(f.unwrap());
        }
        assert!(alloc.allocate_frame().is_none());
        // Deallocate all and reallocate all
        for f in &frames {
            alloc.deallocate_frame(*f);
        }
        let mut seen = Vec::new();
        for _ in 0..n {
            let f = alloc.allocate_frame();
            assert!(f.is_some());
            let f = f.unwrap();
            assert!(!seen.contains(&f));
            seen.push(f);
        }
        assert!(alloc.allocate_frame().is_none());
    }

    #[test]
    fn freelist_allocator_reuse_order() {
        let start = 0x200000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = alloc.allocate_frame().unwrap();
        let f2 = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(f1);
        alloc.deallocate_frame(f2);
        // Should get f2 first (LIFO)
        let r1 = alloc.allocate_frame().unwrap();
        let r2 = alloc.allocate_frame().unwrap();
        assert_eq!(r1, f2);
        assert_eq!(r2, f1);
    }
}
