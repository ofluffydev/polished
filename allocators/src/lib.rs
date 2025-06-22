#![no_std]

//! # allocators
//!
//! A collection of simple, fast, and well-tested memory allocator implementations for use in kernels, embedded, and low-level systems.
//!
//! ## Provided Allocators
//!
//! - [`bump`]: Bump allocators for fast, linear allocation with no individual deallocation.
//! - [`stack`]: Stack allocator for LIFO (last-in, first-out) allocation and deallocation.
//! - [`frame`]: Physical frame allocators for managing fixed-size memory frames (e.g., for paging).
//!
//! ## When to Use
//!
//! - Use a **bump allocator** for temporary arenas, parsing, or when all allocations can be freed at once.
//! - Use a **stack allocator** for temporary workspaces or when allocations and deallocations occur in strict LIFO order.
//! - Use a **frame allocator** for physical memory management in kernels or hypervisors.
//!
//! ## Features
//!
//! - Suitable for `no_std` environments
//! - Thread-safe allocation (where applicable)
//! - Minimal and predictable overhead
//! - Extensively tested for correctness, alignment, OOM, and safety contracts
//!
//! ## Safety
//!
//! All allocators require that the allocator outlives all allocations, and that pointers are not used after reset or drop. See individual module docs for details.
//!
//! ## Modules
//!
//! - [`bump`] — Bump allocators (heap-backed and static)
//! - [`stack`] — Stack allocator (LIFO discipline)
//! - [`frame`] — Physical frame allocators (bump and free-list)
//!
//! ## Example
//!
//! ```rust
//! use polished_allocators::bump::BumpAllocator;
//! use core::alloc::{Layout, GlobalAlloc};
//!
//! let alloc = BumpAllocator::new(1024);
//! let layout = Layout::from_size_align(16, 8).unwrap();
//! let ptr = unsafe { GlobalAlloc::alloc(&alloc, layout) };
//! assert!(!ptr.is_null());
//! // ...
//! alloc.reset();
//! ```
//!
//! See each module for more details and usage examples.

extern crate alloc;

/// Bump allocator module: provides fast, linear allocation with no individual deallocation.
pub mod bump;
/// Frame allocator module: provides physical frame allocators for managing fixed-size memory frames.
pub mod frame;
/// Stack allocator module: provides LIFO (last-in, first-out) allocation and deallocation.
pub mod stack;
