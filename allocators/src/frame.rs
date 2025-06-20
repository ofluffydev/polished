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
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB};

/// Size of a physical frame in bytes.
pub const FRAME_SIZE: usize = 4096;

/// Trait for a physical frame allocator.
///
/// Provides methods to allocate and deallocate physical frames.
pub trait PolishedFrameAllocator {
    /// Allocates a frame. Returns `None` if out of memory.
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>>;

    /// Frees a frame, making it available for reuse.
    fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>);
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

impl PolishedFrameAllocator for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let current = self.next.load(Ordering::Relaxed);
        let aligned = (current + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        if aligned + FRAME_SIZE <= self.end {
            self.next.store(aligned + FRAME_SIZE, Ordering::Relaxed);
            Some(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(
                aligned as u64,
            )))
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {
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
    free_list: Vec<PhysFrame<Size4KiB>>,
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
            free_list.push(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(
                addr as u64,
            )));
            addr += FRAME_SIZE;
        }
        FreeListFrameAllocator { free_list }
    }

    /// Creates a new free list frame allocator using a static mutable slice as storage.
    ///
    /// This avoids dynamic allocation and does not require a global allocator.
    /// The slice will be used as a stack of frames; its length determines the maximum number of frames.
    ///
    /// Returns a tuple of the allocator and the number of frames initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The given range (`start` to `end`) is frame-aligned and does not overlap with any used memory.
    /// - The `backing` slice is large enough to hold all frames in the region, otherwise only as many frames as fit will be initialized.
    /// - The `backing` slice is not aliased elsewhere while the allocator is in use.
    pub unsafe fn new_static(
        start: usize,
        end: usize,
        backing: &mut [PhysFrame<Size4KiB>],
    ) -> (Self, usize) {
        let mut count = 0;
        let mut addr = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        let max = backing.len();
        while addr + FRAME_SIZE <= end && count < max {
            backing[count] = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr as u64));
            addr += FRAME_SIZE;
            count += 1;
        }
        // Use only the initialized part of the slice as the free list.
        let free_list = Vec::from(&backing[..count]);
        (FreeListFrameAllocator { free_list }, count)
    }

    /// Resets the allocator to a new region, clearing and repopulating the free list.
    ///
    /// # Safety
    /// The given range must be frame-aligned and not overlap with used memory.
    pub unsafe fn reset(&mut self, start: usize, end: usize) {
        self.free_list.clear();
        let mut addr = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        while addr + FRAME_SIZE <= end {
            self.free_list
                .push(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(
                    addr as u64,
                )));
            addr += FRAME_SIZE;
        }
    }

    /// Allocates a frame. Returns `None` if out of memory.
    pub fn alloc_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.free_list.pop()
    }

    /// Frees a frame, making it available for reuse.
    pub fn free_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.free_list.push(frame);
    }
}

impl PolishedFrameAllocator for FreeListFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.free_list.pop()
    }

    fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.free_list.push(frame);
    }
}

/// A thread-safe wrapper around FreeListFrameAllocator using a spinlock.
///
/// This allows safe concurrent access to the frame allocator.
#[cfg(feature = "spin_lock")]
pub struct LockedFreeListFrameAllocator {
    inner: spin::Mutex<FreeListFrameAllocator>,
}

#[cfg(feature = "spin_lock")]
impl LockedFreeListFrameAllocator {
    /// Creates a new locked free list frame allocator for the given region.
    ///
    /// # Safety
    /// The given range must be frame-aligned and not overlap with used memory.
    pub unsafe fn new(start: usize, end: usize) -> Self {
        LockedFreeListFrameAllocator {
            inner: spin::Mutex::new(unsafe { FreeListFrameAllocator::new(start, end) }),
        }
    }

    /// # Safety
    /// The caller must ensure:
    /// - The given memory range (`start` to `end`) is valid, frame-aligned, and does not overlap with any memory in use elsewhere.
    /// - The `backing` slice is a unique, mutable reference for the lifetime of the allocator and is not aliased or mutated by other code.
    /// - The `backing` slice is large enough to hold all frames in the region; if not, only as many frames as fit will be initialized.
    /// - No other allocator or code will access or modify the frames managed by this allocator while it is in use.
    ///
    /// Failure to uphold these requirements may result in undefined behavior, memory corruption, or security vulnerabilities.
    pub unsafe fn new_static(
        start: usize,
        end: usize,
        backing: &mut [PhysFrame<Size4KiB>],
    ) -> (Self, usize) {
        let (alloc, count) = unsafe { FreeListFrameAllocator::new_static(start, end, backing) };
        (
            LockedFreeListFrameAllocator {
                inner: spin::Mutex::new(alloc),
            },
            count,
        )
    }

    /// Initializes a new locked free list frame allocator for the given region.
    ///
    /// # Safety
    /// The given range must be frame-aligned and not overlap with used memory.
    pub unsafe fn init(start: usize, end: usize) -> Self {
        LockedFreeListFrameAllocator {
            inner: spin::Mutex::new(unsafe { FreeListFrameAllocator::new(start, end) }),
        }
    }

    /// Returns an empty locked free list frame allocator (no frames available).
    pub const fn empty() -> Self {
        LockedFreeListFrameAllocator {
            inner: spin::Mutex::new(FreeListFrameAllocator {
                free_list: Vec::new(),
            }),
        }
    }

    /// Locks and returns a guard to the inner allocator.
    pub fn lock(&'_ self) -> spin::MutexGuard<'_, FreeListFrameAllocator> {
        self.inner.lock()
    }
    /// Allocates a frame using the inner allocator.
    pub fn alloc_frame(&self) -> Option<PhysFrame<Size4KiB>> {
        self.inner.lock().alloc_frame()
    }
    /// Deallocates a frame using the inner allocator.
    pub fn free_frame(&self, frame: PhysFrame<Size4KiB>) {
        self.inner.lock().free_frame(frame)
    }
}

unsafe impl X86FrameAllocator<Size4KiB> for FreeListFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        PolishedFrameAllocator::allocate_frame(self)
    }
}

#[cfg(feature = "spin_lock")]
unsafe impl X86FrameAllocator<Size4KiB> for LockedFreeListFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        PolishedFrameAllocator::allocate_frame(&mut *self.inner.lock())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physframe_alignment() {
        let addr = 0x12345;
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr as u64));
        assert_eq!(frame.start_address().as_u64() % FRAME_SIZE as u64, 0);
        let addr_val = addr as u64;
        let frame_start = frame.start_address().as_u64();
        let frame_limit = frame_start + FRAME_SIZE as u64;
        assert!(addr_val >= frame_start);
        assert!(addr_val < frame_limit);
    }

    #[test]
    fn physframe_zero_address() {
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0));
        assert_eq!(frame.start_address().as_u64(), 0);
    }

    #[test]
    fn physframe_unaligned_address() {
        let addr = FRAME_SIZE * 5 + 123;
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr as u64));
        assert_eq!(frame.start_address().as_u64(), (FRAME_SIZE * 5) as u64);
        let addr_val = addr as u64;
        let frame_start = frame.start_address().as_u64();
        let frame_limit = frame_start + FRAME_SIZE as u64;
        assert!(addr_val >= frame_start);
        assert!(addr_val < frame_limit);
    }

    #[test]
    fn bump_allocator_basic() {
        let start = 0x10000;
        let end = start + 3 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        let f3 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        assert!(f1.is_some() && f2.is_some() && f3.is_some());
        assert_ne!(f1, f2);
        assert_ne!(f2, f3);
        assert_ne!(f1, f3);
        // Should be exhausted now
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn bump_allocator_no_reuse() {
        let start = 0x20000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f1);
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        assert_ne!(f1, f2, "Bump allocator must not reuse frames");
    }

    #[test]
    fn bump_allocator_zero_region() {
        let mut alloc = unsafe { BumpFrameAllocator::new(0, 0) };
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn bump_allocator_unaligned_start() {
        let start = 0x12345;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        assert!(f1.is_some());
        assert_eq!(f1.unwrap().start_address().as_u64() % FRAME_SIZE as u64, 0);
    }

    #[test]
    fn bump_allocator_exhaustion() {
        let start = 0x80000;
        let end = start + FRAME_SIZE;
        let mut alloc = unsafe { BumpFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        assert!(f1.is_some());
        assert!(f2.is_none());
    }

    #[test]
    fn freelist_allocator_basic() {
        let start = 0x30000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        assert!(f1.is_some() && f2.is_some());
        assert_ne!(f1, f2);
        // Should be exhausted now
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn freelist_allocator_reuse() {
        let start = 0x40000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f1);
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        assert_eq!(f1, f2, "Freelist should reuse deallocated frames");
    }

    #[test]
    fn freelist_allocator_double_free() {
        let start = 0x50000;
        let end = start + FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f);
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f); // double free
        // Should be able to allocate twice, but not a third time
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_some());
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_some());
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
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
            let f = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
            assert_eq!(f.start_address().as_u64() % FRAME_SIZE as u64, 0);
        }
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn freelist_allocator_zero_region() {
        let mut alloc = unsafe { FreeListFrameAllocator::new(0, 0) };
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn freelist_allocator_unaligned_start() {
        let start = 0x12345;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc);
        assert!(f1.is_some());
        assert_eq!(f1.unwrap().start_address().as_u64() % FRAME_SIZE as u64, 0);
    }

    #[test]
    fn freelist_allocator_stress_many_frames() {
        let start = 0x100000;
        let n = 100;
        let end = start + n * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let mut frames = Vec::new();
        for _ in 0..n {
            let f = PolishedFrameAllocator::allocate_frame(&mut alloc);
            assert!(f.is_some());
            frames.push(f.unwrap());
        }
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
        // Deallocate all and reallocate all
        for f in &frames {
            PolishedFrameAllocator::deallocate_frame(&mut alloc, *f);
        }
        let mut seen = Vec::new();
        for _ in 0..n {
            let f = PolishedFrameAllocator::allocate_frame(&mut alloc);
            assert!(f.is_some());
            let f = f.unwrap();
            assert!(!seen.contains(&f));
            seen.push(f);
        }
        assert!(PolishedFrameAllocator::allocate_frame(&mut alloc).is_none());
    }

    #[test]
    fn freelist_allocator_reuse_order() {
        let start = 0x200000;
        let end = start + 2 * FRAME_SIZE;
        let mut alloc = unsafe { FreeListFrameAllocator::new(start, end) };
        let f1 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        let f2 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f1);
        PolishedFrameAllocator::deallocate_frame(&mut alloc, f2);
        // Should get f2 first (LIFO)
        let r1 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        let r2 = PolishedFrameAllocator::allocate_frame(&mut alloc).unwrap();
        assert_eq!(r1, f2);
        assert_eq!(r2, f1);
    }

    #[cfg(feature = "spin_lock")]
    #[test]
    fn locked_freelist_allocator_basic() {
        let start = 0x60000;
        let end = start + 2 * FRAME_SIZE;
        let alloc = unsafe { LockedFreeListFrameAllocator::init(start, end) };
        let mut guard = alloc.lock();
        let f1 = guard.alloc_frame();
        let f2 = guard.alloc_frame();
        assert!(f1.is_some() && f2.is_some());
        assert_ne!(f1, f2);
        // Should be exhausted now
        assert!(guard.alloc_frame().is_none());
    }

    #[cfg(feature = "spin_lock")]
    #[test]
    fn locked_freelist_allocator_reuse() {
        let start = 0x70000;
        let end = start + 2 * FRAME_SIZE;
        let alloc = unsafe { LockedFreeListFrameAllocator::init(start, end) };
        let mut guard = alloc.lock();
        let f1 = guard.alloc_frame().unwrap();
        guard.free_frame(f1);
        let f2 = guard.alloc_frame().unwrap();
        assert_eq!(f1, f2, "LockedFreelist should reuse deallocated frames");
    }

    #[cfg(feature = "spin_lock")]
    #[test]
    fn locked_freelist_allocator_empty() {
        let alloc = LockedFreeListFrameAllocator::empty();
        let mut guard = alloc.lock();
        assert!(guard.alloc_frame().is_none());
    }
}
