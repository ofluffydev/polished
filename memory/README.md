# Polished Memory Library

**Polished Memory** is a Rust library providing fundamental memory manipulation routines—`memset`, `memcmp`, `memcpy`, and `memmove`—for use in `no_std` environments such as kernels, bootloaders, or embedded systems. It is a core component of the Polished OS project, ensuring that essential low-level memory operations are available even when the Rust standard library is not.

______________________________________________________________________

## What Does This Library Do?

This crate implements the following C-style memory functions:

- **`memset`**: Sets a block of memory to a specific byte value.
- **`memcmp`**: Compares two blocks of memory byte by byte.
- **`memcpy`**: Copies a block of memory from one location to another (non-overlapping).
- **`memmove`**: Copies a block of memory from one location to another, correctly handling overlapping regions.

All functions are marked with `#[no_mangle]` and use C ABI (`extern "C"`), making them available to both Rust and C code, and ensuring the correct symbol names are exported for the linker.

______________________________________________________________________

## Why Is This Needed?

In `no_std` Rust environments, the standard library is unavailable—including its implementations of these essential memory routines. However, the Rust compiler and core library expect these symbols to exist, as they are used for:

- **Compiler-generated code**: The Rust compiler may emit calls to these functions for certain operations (e.g., struct initialization, slice copying, zeroing memory).
- **Core library requirements**: The `core` crate, which is always linked in `no_std` projects, assumes these functions are present for low-level operations.
- **C interoperability**: If you link C code or use C FFI, these functions are required for compatibility.

If these symbols are missing, linking will fail or runtime errors may occur. By providing them, this crate ensures that Rust code (and any C code linked in) can safely and efficiently perform basic memory operations, even in bare-metal or OS development contexts.

______________________________________________________________________

## How Does Rust Use These Functions?

- **Zeroing and copying**: When you use constructs like `let x = [0u8; 1024];` or `dst.copy_from_slice(src)`, the compiler may generate calls to `memset` or `memcpy`.
- **Slice and array operations**: Many methods on slices and arrays, such as `clone_from_slice`, rely on these routines for performance and correctness.
- **Panic and unwinding**: Some panic or unwinding code paths may use these routines to clean up memory.
- **FFI and core intrinsics**: The Rust core library and FFI code expect these functions to be available for interoperability with C and other languages.

______________________________________________________________________

## Safety

All functions in this crate are `unsafe` and require the caller to uphold strict invariants regarding pointer validity, alignment, and region overlap. Incorrect usage can lead to undefined behavior, memory corruption, or security vulnerabilities. See each function's documentation for details.

______________________________________________________________________

## Usage

Add this crate to your `no_std` project to satisfy the compiler's requirements for these memory routines. You may also use these functions directly if needed:

```rust
// Example: Zero a buffer
unsafe {
    memory::memset(buf.as_mut_ptr(), 0, buf.len());
}
```

Typically, you do not need to call these functions directly—Rust and the core library will use them automatically as needed.

______________________________________________________________________

## Implementation Notes

- All functions are implemented in safe Rust where possible, but marked `unsafe` due to raw pointer manipulation.
- The signatures and symbol names match the C standard library for maximum compatibility.
- The crate is `#![no_std]` and suitable for bare-metal or OS development.

______________________________________________________________________

## Further Reading

- [Rustonomicon: Unstable Library Features - mem\* functions](https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-unsafe-code)
- [OSDev Wiki: C Standard Library - Memory Functions](https://wiki.osdev.org/Category:C_Library)
- [core::ptr documentation](https://doc.rust-lang.org/core/ptr/index.html)

______________________________________________________________________

## License

This library is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root of the repository for details.

______________________________________________________________________

**Polished Memory** is part of the [Polished OS](../README.md) project. Contributions and feedback are welcome!
