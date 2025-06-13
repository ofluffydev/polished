# Polished allocators

**This crate is part of [Project Polished](../README.md), a modular, modern, and inclusive operating system foundation and toolkit written in Rust. Project Polished provides reusable, well-documented components for OS development, and this `allocators` crate supplies foundational memory allocator implementations for use in kernels, bootloaders, and embedded systems.**

## What is an allocator?

An allocator is a component responsible for managing memory in a program or operating system. It provides mechanisms to allocate and deallocate blocks of memory at runtime, typically from a fixed pool or the system's available memory. Allocators are fundamental in low-level systems programming, where dynamic memory management is required but standard library allocators may not be available or suitable (such as in kernels, bootloaders, or embedded systems).

Allocators differ in their strategies for handling memory requests, trade-offs between speed and flexibility, and how/when memory is reclaimed. Choosing the right allocator depends on the use case, allocation patterns, and system constraints.

## Types of allocators and allocation APIs

Memory allocation is a broad topic, and different programming languages and environments use different allocators and APIs. Here are some common terms, what they mean, and where they are defined or used in code:

- **malloc/free**: The classic C standard library functions for dynamic memory allocation. `malloc` and `free` are declared in `<stdlib.h>`, and implemented by the system's libc (e.g., glibc, musl). The actual allocator (e.g., `ptmalloc`, `jemalloc`, `dlmalloc`) is chosen by the platform. In C/C++ code, you use `malloc`/`free` directly; in Rust, you can call them via the `libc` crate or FFI.

- **OS allocators**: Operating systems provide low-level primitives for memory management, such as `VirtualAlloc` (Windows), `mmap` (Unix), or `sbrk`. These are system calls, typically declared in system headers (e.g., `<sys/mman.h>` for `mmap`). User-space allocators like `malloc` ultimately rely on these to obtain large memory regions. In kernel or OS code, you may call these directly.

- **global_allocator (Rust)**: In Rust, the `#[global_allocator]` attribute specifies the allocator used for heap allocations (e.g., `Box`, `Vec`). By default, this is the system allocator (using `malloc`/`free`), but you can provide your own by implementing the `core::alloc::GlobalAlloc` trait and marking a static instance with `#[global_allocator]`. See [Rust docs](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) for details. In this crate, custom allocators like `BumpAllocator` and `StackAllocator` implement `GlobalAlloc` in their respective files.

- **pymalloc**: Python's specialized allocator for small objects, implemented in the CPython runtime (see `Objects/obmalloc.c`). Python code does not call this directly; it's used internally for object allocation. Other languages (e.g., Java, Go) have their own runtime allocators, usually implemented in the language runtime source code.

- **arena allocators**: Arena (or region) allocators are often implemented as a struct or class that owns a large buffer and provides fast, linear allocation. In Rust, you might see crates like `typed-arena` or custom arena types. In this crate, the `BumpAllocator` is an example of an arena-style allocator (see `src/bump.rs`).

- **slab/pool allocators**: These are implemented as data structures that manage pools of fixed-size objects. In Rust, you might use the `slab` crate or implement your own. In kernel code, slab allocators are often part of the memory management subsystem (e.g., Linux's `mm/slab.c`).

- **Garbage-collected allocators**: Languages like Java, Go, and JavaScript use garbage collectors, implemented in their runtimes (e.g., OpenJDK's GC, Go's runtime, V8 for JS). You don't call these directly; memory is managed automatically.

- **Custom allocators**: In low-level or performance-critical code (kernels, games, embedded), custom allocators are implemented as structs/classes with allocation methods, or by implementing a language-specific allocator trait/interface. In Rust, implement `GlobalAlloc` or use the `Allocator` trait (nightly). In this crate, see `src/bump.rs` and `src/stack.rs` for custom allocator implementations.

In summary, allocators are defined and used at different layers:

- **System/library level:** `malloc`, OS syscalls, runtime allocators
- **Language/runtime level:** `#[global_allocator]` in Rust, Python/Java/Go runtimes
- **Application/crate level:** Custom allocators (like those in this crate), arena/slab types, or wrappers around system allocators

Understanding where an allocator is defined and how it is used in code is key to choosing or implementing the right one for your needs.

## Implemented Allocators

### Boot-time allocator (early bump/stack allocator)

#### Bump Allocator

A bump allocator allocates memory linearly from a fixed-size heap. Each allocation simply advances a pointer (the "bump"), and individual deallocations are not supported—memory is only reclaimed when the allocator is reset or dropped.

- **How it works:**
  - Keeps an offset into a preallocated buffer.
  - Each allocation returns a pointer to the next aligned address, bumping the offset forward.
  - No per-allocation metadata; no deallocation (except for a full reset).
- **Use cases:**
  - Early boot memory allocation (before a full allocator is available)
  - Temporary or short-lived allocations where all memory can be reclaimed at once
  - Embedded or `no_std` environments
- **Pros:**
  - Extremely fast (just pointer arithmetic)
  - Simple implementation
  - No fragmentation
- **Cons:**
  - Cannot free individual allocations
  - Memory is only reclaimed on reset or drop
  - Not suitable for long-lived or complex allocation patterns

#### Stack Allocator

A stack allocator also uses a fixed-size buffer, but supports LIFO (last-in, first-out) deallocation. Only the most recent allocation can be freed, enforcing a stack discipline.

- **How it works:**
  - Each allocation pushes a header with the previous offset onto the stack.
  - Deallocation pops the most recent allocation, restoring the previous offset.
  - Only the last allocation can be freed (LIFO order).
- **Use cases:**
  - Temporary allocations with strict LIFO usage (e.g., recursive algorithms, function call frames)
  - Boot-time or kernel subsystems with predictable allocation patterns
- **Pros:**
  - Fast allocation and deallocation
  - Simple, with minimal metadata
  - No fragmentation if used correctly
- **Cons:**
  - Only supports freeing the most recent allocation
  - Not suitable for arbitrary allocation/free patterns

#### Physical page/frame allocator

A physical frame allocator manages physical memory in fixed-size frames (pages), which is essential for paging, DMA, and device buffers. This crate provides both a bump frame allocator and a free-list frame allocator:

- **How it works:**
  - The bump frame allocator hands out frames linearly from a region, never reusing freed frames (no deallocation).
  - The free-list frame allocator maintains a list of free frames and supports allocation and deallocation (reusing frames).
- **Use cases:**
  - Kernel physical memory management
  - Paging, DMA, device buffers
- **Pros:**
  - Simple and fast (bump)
  - Supports reuse and deallocation (free-list)
- **Cons:**
  - Bump allocator cannot reuse frames
  - Free-list allocator requires additional bookkeeping

See `src/frame.rs` for implementation details and tests.

## Planned Allocators

- **Slab/pool allocator:** For small or fixed-size kernel objects. Reduces fragmentation and allocation overhead.
- **DMA/contiguous allocator:** For device buffers (e.g., VirtIO), ensures physically contiguous memory.
- **User-space allocator:** Isolated heaps for user processes, supporting allocation and deallocation.
- **Buddy system allocator:** For efficient allocation and deallocation of memory blocks of power-of-two sizes, reducing fragmentation.
- **Linked list allocator:** A simple general-purpose heap allocator using a free-list of variable-sized blocks, suitable for kernel heaps.

## Status

- [x] Boot-time allocator (bump/stack)
- [ ] Buddy system allocator
- [ ] Linked list allocator
- [x] Physical page/frame allocator
- [ ] Slab/pool allocator
- [ ] DMA/contiguous allocator
- [ ] User-space allocator
