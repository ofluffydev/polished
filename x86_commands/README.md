# x86_commands

**x86_commands** is a low-level Rust library providing direct access to x86 and x86_64 CPU instructions and hardware operations, specifically designed for use in operating system kernels, bootloaders, and other bare-metal environments. This crate is part of the [Polished OS](../README.md) project, but is modular and can be reused in other OS development efforts.

______________________________________________________________________

## Purpose and Scope

This crate aims to offer a minimal, well-documented, and safe (where possible) set of helpers for interacting with x86-family CPUs. It currently provides only a handful of helpers, but is intended to grow into a comprehensive alternative to crates like [`x86_64`](https://docs.rs/x86_64/) or [`x86`](https://docs.rs/x86/), tailored for the needs of Polished OS and similar projects.

**Key goals:**

- Expose essential CPU instructions and port I/O operations for kernel/bootloader use
- Serve as a foundation for higher-level abstractions in Polished OS
- Remain modular and replaceable, with minimal dependencies
- Provide clear documentation and usage examples

______________________________________________________________________

## Current Features

- **PIC (Programmable Interrupt Controller) helpers:**
  - `disable_pic()`: Masks all interrupts from the legacy PIC (8259), a common step before enabling APIC in modern kernels.
- **Inline assembly wrappers:**
  - All functions use `core::arch::asm!` for direct hardware access.

*Note: The crate currently only contains a few helpers, but is designed to expand as Polished OS evolves. Planned features include port I/O, CPU feature detection, MSR access, and more.*

______________________________________________________________________

## Example Usage

```rust
use x86_commands::disable_pic;

disable_pic(); // Masks all legacy PIC interrupts (required before enabling APIC)
```

All functions are `no_std` and require a privileged (kernel or bootloader) context. They are not safe for use in userspace or general application code.

______________________________________________________________________

## OS Development Context

This crate is intended for use in the earliest stages of OS development, where direct hardware access is required and no standard library is available. It is especially useful for:

- Writing custom kernels (e.g., Polished OS)
- Developing UEFI or BIOS bootloaders
- Building hypervisors or bare-metal applications

**Why not use `x86_64` or `x86` crates?**

- `x86_commands` is designed to be minimal, with only the features needed for Polished OS
- It can be extended or replaced as the project grows
- The API and implementation are tailored for modern Rust idioms and safety

______________________________________________________________________

## Safety and Portability

- All functions are `unsafe` or require `unsafe` blocks, as they interact directly with hardware
- Only works on x86 or x86_64 CPUs; will not compile or run on other architectures
- Intended for use in kernel or bootloader code only

______________________________________________________________________

## Planned Expansion

As Polished OS matures, this crate may:

- Add more CPU instructions (e.g., `hlt`, `cli`, `sti`, `cpuid`, etc.)
- Provide port I/O helpers (`inb`, `outb`, etc.)
- Support model-specific register (MSR) access
- Offer higher-level abstractions for interrupts, paging, and more
- Eventually replace the need for external crates like `x86_64` in Polished OS

______________________________________________________________________

## Contributing

Contributions are welcome! If you have suggestions, bug reports, or want to add new helpers, please open an issue or pull request.

______________________________________________________________________

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the [LICENSE](../LICENSE) file for details.

______________________________________________________________________

## References & Inspiration

- [x86_64 crate](https://docs.rs/x86_64/)
- [x86 crate](https://docs.rs/x86/)
- [Polished OS](../README.md)
- [Rust OSDev community](https://osdev.rs/)

______________________________________________________________________

*This crate is under active development as part of the Polished OS project. Its API and features may change as the OS evolves.*
