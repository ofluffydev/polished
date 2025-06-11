# Polished OS Bootloader

The **Polished OS Bootloader** is a UEFI-based bootloader written in Rust. It is responsible for initializing the UEFI environment, loading the Polished OS kernel from disk, setting up the graphics framebuffer, and transferring control to the kernel. This component is a critical part of the [Polished OS](../README.md) project, which aims to provide a modern, modular, and accessible operating system foundation in Rust.

______________________________________________________________________

## Overview

The bootloader is implemented as a UEFI application using the [`uefi-rs`](https://github.com/rust-osdev/uefi-rs) crate. It leverages UEFI services to:

- Load the kernel binary (in ELF format) from the EFI system partition
- Set up a graphics framebuffer and pass its configuration to the kernel
- Output status and diagnostic messages to the UEFI console
- Transfer control to the kernel's entry point, passing framebuffer info as an argument

This approach allows the bootloader to remain portable and hardware-agnostic, relying on UEFI's standardized interfaces for file access, graphics, and console output.

______________________________________________________________________

## How It Works

1. **UEFI Initialization**: The bootloader initializes the UEFI environment and clears the screen. Optionally, it displays a greeting message for user feedback.
1. **Kernel Loading**: Using UEFI file protocols, the bootloader loads the kernel binary (typically located at `\EFI\BOOT\kernel`) from the EFI system partition. The kernel must be in ELF format.
1. **Framebuffer Setup**: The bootloader initializes the graphics framebuffer using UEFI graphics protocols. It collects framebuffer configuration details (resolution, address, pixel format) and prepares them to be passed to the kernel.
1. **Transfer of Control**: The bootloader uses inline assembly to jump to the kernel's entry point, passing a pointer to the framebuffer configuration as the first argument (in `rdi`). After this point, the bootloader's execution ends and the kernel takes over.

### Code Structure

- `src/main.rs`: UEFI application entry point. Calls into the bootloader library to initialize UEFI and boot the system.
- `src/lib.rs`: Main bootloader logic, including UEFI setup, kernel loading, framebuffer initialization, and transfer of control.
- Relies on workspace crates:
  - `elf_loader`: Loads ELF binaries from disk
  - `graphics`: Framebuffer and graphics primitives
  - `uefi`: UEFI protocol bindings and helpers

______________________________________________________________________

## Building

The bootloader is built as part of the Polished OS workspace. You do not need to build it separately. To build the bootloader and kernel together, run from the workspace root:

```sh
make check-artifacts
```

Or to build and run the full OS in QEMU:

```sh
make run
```

See the [workspace README](../README.md) for more details on prerequisites and build instructions.

______________________________________________________________________

## License

The bootloader is licensed under the [zlib License](https://zlib.net/zlib_license.html):

```
Copyright (c) 2025

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
   claim that you wrote the original software. If you use this software
   in a product, an acknowledgment in the product documentation would be
   appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
   misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.
```

______________________________________________________________________

## Acknowledgments

- [uefi-rs](https://github.com/rust-osdev/uefi-rs) — UEFI support in Rust
- [buddy_system_allocator](https://github.com/rcore-os/buddy_system_allocator) — Heap allocator (used by the kernel)
- Rust OSDev community — For resources, examples, and inspiration

______________________________________________________________________

For more information about Polished OS, see the [main README](../README.md).
