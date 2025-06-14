// Buddy System Allocator for memory management in Rust.
//
// This file implements a simple buddy allocator, which is a memory allocation algorithm that divides memory into partitions to try to satisfy a memory request as suitably as possible. It is efficient for splitting and merging blocks of memory, making it suitable for systems programming and OS kernels.
//
// The buddy allocator works by splitting memory into blocks of size 2^k, and merging adjacent free blocks (buddies) when possible. This helps reduce fragmentation and makes allocation/deallocation fast.

use alloc::vec;
use alloc::vec::Vec;
use core::ptr;

/// Trait for a generic buddy system allocator.
///
/// This trait defines the basic interface for a buddy system allocator, which can allocate, deallocate, and reset memory blocks.
pub trait BuddySystemAllocator {
    /// Allocates a block of memory of the specified size (in bytes).
    ///
    /// Returns a pointer to the allocated memory, or `None` if allocation fails.
    ///
    /// # Arguments
    /// * `size` - The size of the memory block to allocate, in bytes.
    fn alloc(&mut self, size: usize) -> Option<*mut u8>;

    /// Deallocates a previously allocated block of memory.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the memory block to deallocate. Must be a pointer returned by `alloc`.
    /// * `size` - The size of the memory block to deallocate. Must match the size used in `alloc`.
    fn dealloc(&mut self, ptr: *mut u8, size: usize);

    /// Resets the allocator, freeing all allocated blocks and returning to the initial state.
    fn reset(&mut self);
}

/// BuddyAllocator struct manages a contiguous region of memory using the buddy system algorithm.
///
/// # Fields
/// * `total_size` - The total size of the managed memory region (in bytes).
/// * `min_block_size` - The minimum size of a block (in bytes). Must be a power of two.
/// * `max_order` - The maximum order (log2 of the number of min blocks in total_size).
/// * `base_ptr` - Pointer to the start of the managed memory region.
/// * `free_lists` - A vector of vectors, where each inner vector is a free list for blocks of a given order.
pub struct BuddyAllocator {
    total_size: usize,             // Total size of the managed memory region
    min_block_size: usize,         // Minimum block size (must be power of two)
    max_order: usize,              // Maximum order (number of splits possible)
    base_ptr: *mut u8,             // Pointer to the start of the memory region
    free_lists: Vec<Vec<*mut u8>>, // Free lists for each order (block size)
}

impl BuddyAllocator {
    /// Creates a new BuddyAllocator with the specified total size and minimum block size.
    ///
    /// # Arguments
    /// * `total_size` - The total size of the memory region to manage (in bytes).
    /// * `min_block_size` - The minimum block size (in bytes). Must be a power of two.
    ///
    /// # Panics
    /// Panics if `min_block_size` is not a power of two or if `total_size` is less than `min_block_size`.
    pub fn new(total_size: usize, min_block_size: usize) -> Self {
        assert!(
            min_block_size.is_power_of_two(),
            "min_block_size must be a power of two"
        );
        assert!(
            total_size >= min_block_size,
            "total_size must be >= min_block_size"
        );
        // Calculate the maximum order: how many times can we double min_block_size to reach total_size
        let max_order = (total_size / min_block_size).trailing_zeros() as usize;
        BuddyAllocator {
            total_size,
            min_block_size,
            max_order,
            base_ptr: ptr::null_mut(),
            free_lists: vec![Vec::new(); max_order + 1],
        }
    }

    /// Initializes the buddy allocator, allocating the memory region and setting up the initial free list.
    ///
    /// This must be called before any allocations are made. It allocates a contiguous region of memory and places a single block in the largest free list.
    ///
    /// # Returns
    /// * `true` if initialization succeeded, `false` if allocation failed.
    pub fn init(&mut self) -> bool {
        // Create a memory layout for the total size, aligned to min_block_size
        let layout =
            match alloc::alloc::Layout::from_size_align(self.total_size, self.min_block_size) {
                Ok(layout) => layout,
                Err(_) => {
                    self.base_ptr = core::ptr::null_mut();
                    return false;
                }
            };
        // Allocate the memory region
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if !ptr.is_null() {
            self.base_ptr = ptr;
            // Place the entire region as a single free block in the largest order
            self.free_lists[self.max_order].push(ptr);
            true
        } else {
            self.base_ptr = core::ptr::null_mut();
            false
        }
    }

    /// Determines the order (power of two) needed to fit a block of the given size.
    ///
    /// # Arguments
    /// * `size` - The size of the block to allocate (in bytes).
    ///
    /// # Returns
    /// * `Some(order)` if the size can be satisfied, `None` if too large.
    fn order_for_size(&self, size: usize) -> Option<usize> {
        let mut order = 0;
        let mut block_size = self.min_block_size;
        // Increase order until block_size >= requested size
        while block_size < size {
            block_size <<= 1;
            order += 1;
        }
        if order > self.max_order {
            None
        } else {
            Some(order)
        }
    }

    /// Returns the block size (in bytes) for a given order.
    ///
    /// # Arguments
    /// * `order` - The order (number of splits from min_block_size).
    fn block_size(&self, order: usize) -> usize {
        self.min_block_size << order
    }

    /// Computes the buddy address for a given pointer and order.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the block.
    /// * `order` - The order of the block.
    ///
    /// # Returns
    /// * Pointer to the buddy block.
    fn buddy_of(&self, ptr: *mut u8, order: usize) -> *mut u8 {
        assert!(
            !self.base_ptr.is_null(),
            "BuddyAllocator not initialized: base_ptr is null"
        );
        let base = self.base_ptr as usize;
        let ptr = ptr as usize;
        if ptr < base {
            panic!(
                "Pointer passed to buddy_of is before base_ptr: ptr={:#x}, base_ptr={:#x}",
                ptr, base
            );
        }
        let offset = ptr - base;
        // XOR with block size to get buddy's offset
        let buddy_offset = offset ^ self.block_size(order);
        unsafe { self.base_ptr.add(buddy_offset) }
    }
}

impl BuddySystemAllocator for BuddyAllocator {
    /// Allocates a block of memory of at least `size` bytes.
    ///
    /// # Arguments
    /// * `size` - The size of the memory block to allocate (in bytes).
    ///
    /// # Returns
    /// * `Some(ptr)` to the allocated block, or `None` if allocation failed.
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        if self.base_ptr.is_null() {
            return None;
        }
        // Find the smallest order that fits the requested size
        let order = self.order_for_size(size)?;
        // Search for a free block in the free lists, starting from the requested order up to the largest
        for current_order in order..=self.max_order {
            if let Some(&block) = self.free_lists[current_order].last() {
                // Remove block from free list
                self.free_lists[current_order].pop();
                // Split blocks down to requested order
                for split_order in (order..current_order).rev() {
                    // Calculate buddy address for the split
                    let buddy = unsafe { block.add(self.block_size(split_order)) };
                    self.free_lists[split_order].push(buddy);
                }
                return Some(block);
            }
        }
        // No suitable block found
        None
    }

    /// Deallocates a previously allocated block of memory.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the memory block to deallocate.
    /// * `size` - The size of the memory block to deallocate.
    fn dealloc(&mut self, mut ptr: *mut u8, size: usize) {
        if self.base_ptr.is_null() {
            return;
        }
        // Find the order for the given size
        if let Some(mut order) = self.order_for_size(size) {
            // Try to merge with buddy blocks as long as possible
            while order < self.max_order {
                let buddy = self.buddy_of(ptr, order);
                // Check if buddy is free (present in the free list)
                if let Some(pos) = self.free_lists[order].iter().position(|&p| p == buddy) {
                    // Remove buddy from free list and merge
                    self.free_lists[order].remove(pos);
                    // Always keep the lower address as the merged block
                    if buddy < ptr {
                        ptr = buddy;
                    }
                    order += 1;
                } else {
                    // Buddy not free, stop merging
                    break;
                }
            }
            // Place the (possibly merged) block back into the free list
            self.free_lists[order].push(ptr);
        }
    }

    /// Resets the allocator, freeing all allocated blocks and returning to the initial state.
    fn reset(&mut self) {
        // Clear all free lists
        for list in &mut self.free_lists {
            list.clear();
        }
        // Place the entire region as a single free block in the largest order
        if !self.base_ptr.is_null() {
            self.free_lists[self.max_order].push(self.base_ptr);
        }
    }
}

impl Drop for BuddyAllocator {
    /// Drops the allocator, deallocating the managed memory region.
    ///
    /// This is called automatically when the allocator goes out of scope.
    fn drop(&mut self) {
        if !self.base_ptr.is_null() {
            unsafe {
                alloc::alloc::dealloc(
                    self.base_ptr,
                    alloc::alloc::Layout::from_size_align(self.total_size, self.min_block_size)
                        .unwrap(),
                );
            }
            self.base_ptr = core::ptr::null_mut();
        }
        // Clear all free lists
        for list in &mut self.free_lists {
            list.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::vec::Vec;

    /// Test basic allocation and deallocation of a single block.
    #[test]
    fn test_basic_alloc_dealloc() {
        let total_size = 1024;
        let min_block_size = 32;
        let mut allocator = BuddyAllocator::new(total_size, min_block_size);
        assert!(allocator.init(), "BuddyAllocator init failed");
        let ptr = allocator.alloc(64);
        assert!(ptr.is_some(), "Allocation failed");
        let ptr_val = ptr.unwrap() as usize;
        let base_val = allocator.base_ptr as usize;
        assert!(ptr_val >= base_val, "Allocated pointer is before base_ptr");
        allocator.dealloc(ptr.unwrap(), 64);
    }

    /// Test multiple allocations and deallocations.
    #[test]
    fn test_multiple_allocs_and_deallocs() {
        let total_size = 1024;
        let min_block_size = 32;
        let mut allocator = BuddyAllocator::new(total_size, min_block_size);
        assert!(allocator.init(), "BuddyAllocator init failed");
        let mut ptrs = Vec::new();
        for _ in 0..4 {
            let ptr = allocator.alloc(64);
            assert!(ptr.is_some(), "Allocation failed");
            let ptr_val = ptr.unwrap() as usize;
            let base_val = allocator.base_ptr as usize;
            assert!(ptr_val >= base_val, "Allocated pointer is before base_ptr");
            ptrs.push(ptr.unwrap());
        }
        for &ptr in &ptrs {
            allocator.dealloc(ptr, 64);
        }
    }

    /// Test allocation exhaustion (no more memory available).
    #[test]
    fn test_exhaustion() {
        let total_size = 128;
        let min_block_size = 32;
        let mut allocator = BuddyAllocator::new(total_size, min_block_size);
        assert!(allocator.init(), "BuddyAllocator init failed");
        // Allocate two 64-byte blocks, which should fill the heap.
        let a = allocator.alloc(64);
        assert!(a.is_some(), "Allocation of 64 failed");
        let b = allocator.alloc(64);
        assert!(b.is_some(), "Allocation of 64 failed");
        // Next allocation should fail
        assert!(
            allocator.alloc(64).is_none(),
            "Should not be able to allocate more"
        );
        allocator.dealloc(a.unwrap(), 64);
        allocator.dealloc(b.unwrap(), 64);
    }

    /// Test reset functionality (free all blocks).
    #[test]
    fn test_reset() {
        let total_size = 256;
        let min_block_size = 32;
        let mut allocator = BuddyAllocator::new(total_size, min_block_size);
        assert!(allocator.init(), "BuddyAllocator init failed");
        let ptr1 = allocator.alloc(64);
        let ptr2 = allocator.alloc(64);
        assert!(ptr1.is_some() && ptr2.is_some());
        let ptr1_val = ptr1.unwrap() as usize;
        let ptr2_val = ptr2.unwrap() as usize;
        let base_val = allocator.base_ptr as usize;
        assert!(
            ptr1_val >= base_val && ptr2_val >= base_val,
            "Allocated pointer is before base_ptr"
        );
        allocator.reset();
        // After reset, should be able to allocate again
        let ptr3 = allocator.alloc(128);
        assert!(ptr3.is_some());
        let ptr3_val = ptr3.unwrap() as usize;
        assert!(ptr3_val >= base_val, "Allocated pointer is before base_ptr");
    }

    /// Test fragmentation and merging of blocks.
    #[test]
    fn test_fragmentation_and_merge() {
        let total_size = 256;
        let min_block_size = 32;
        let mut allocator = BuddyAllocator::new(total_size, min_block_size);
        assert!(allocator.init(), "BuddyAllocator init failed");
        let a = allocator.alloc(32).unwrap();
        let b = allocator.alloc(32).unwrap();
        let c = allocator.alloc(32).unwrap();
        let d = allocator.alloc(32).unwrap();
        let base_val = allocator.base_ptr as usize;
        assert!(
            (a as usize) >= base_val
                && (b as usize) >= base_val
                && (c as usize) >= base_val
                && (d as usize) >= base_val,
            "Allocated pointer is before base_ptr"
        );
        allocator.dealloc(a, 32);
        allocator.dealloc(c, 32);
        allocator.dealloc(b, 32);
        allocator.dealloc(d, 32);
        // After all deallocs, should be able to allocate a large block
        let big = allocator.alloc(128);
        assert!(big.is_some(), "Allocator did not merge blocks correctly");
        let big_val = big.unwrap() as usize;
        assert!(big_val >= base_val, "Allocated pointer is before base_ptr");
    }
}
