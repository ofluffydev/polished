//! # memory
//!
//! This crate provides fundamental memory manipulation routines (`memset`, `memcmp`, `memcpy`, and `memmove`) for use in `no_std` Rust environments, such as kernels, bootloaders, or embedded systems.
//!
//! ## Why is this needed?
//!
//! In `no_std` environments, the Rust standard library is unavailable, including its implementations of essential C library functions like `memset`, `memcmp`, `memcpy`, and `memmove`. However, the Rust compiler and core library expect these functions to exist, as they are often used for low-level memory operations, optimizations, and code generation. If these symbols are missing, linking will fail or runtime errors may occur.
//!
//! By providing these functions with the correct signatures and `#[no_mangle]` attributes, this crate ensures that Rust code (and any C code linked in) can safely and efficiently perform basic memory operations, even in bare-metal or OS development contexts.
//!
//! ## Safety
//!
//! All functions in this crate are `unsafe` and require the caller to uphold strict invariants regarding pointer validity, alignment, and region overlap. See each function's documentation for details.
//!
//! ## Usage
//!
//! Link this crate into your `no_std` project to satisfy the compiler's requirements for these memory routines. You may also use these functions directly if needed.

#![no_std]

use core::ptr;

/// Sets `count` bytes starting at `dest` to the given `value`.
///
/// # Safety
///
/// - `dest` must be valid for writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if `dest` is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    for _ in 0..count {
        unsafe { ptr::write(ptr, value) };
        ptr = unsafe { ptr.add(1) };
    }
}

/// Compares the first `n` bytes of the memory areas `s1` and `s2`.
///
/// # Safety
///
/// - Both `s1` and `s2` must be valid for reads of `n` bytes.
/// - The memory regions must not overlap with any mutable references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { ptr::read(s1.add(i)) };
        let b = unsafe { ptr::read(s2.add(i)) };
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

/// Copies `count` bytes from `src` to `dest`.
///
/// # Safety
///
/// - Both `dest` and `src` must be valid for reads and writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
    }
}

/// Moves `count` bytes from `src` to `dest`, correctly handling overlapping regions.
///
/// # Safety
///
/// - Both `dest` and `src` must be valid for reads and writes of `count` bytes.
/// - The memory regions must not overlap with any other references for the duration of this call.
/// - Behavior is undefined if either pointer is null or not properly aligned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if dest as usize <= src as usize || dest as usize >= src as usize + count {
        // Non-overlapping regions, can copy forward
        for i in 0..count {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    } else {
        // Overlapping regions, copy backward
        for i in (0..count).rev() {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    }
}
