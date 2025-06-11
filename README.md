# Polished OS (Project Polished)

![Polished OS Logo](./polished-banner.png)

Photo by <a href="https://unsplash.com/@lifelivedinmono?utm_content=creditCopyText&utm_medium=referral&utm_source=unsplash">Gordon Gerard McLean</a> on <a href="https://unsplash.com/photos/a-window-with-rain-drops-on-it-ual-ZkL2IXQ?utm_content=creditCopyText&utm_medium=referral&utm_source=unsplash">Unsplash</a> (With modifications)

**Polished OS** is an experimental operating system built to be accessible, modern, and more than just a personal project. It aims to eventually support `libc` and POSIX standards, making it suitable for general-purpose use.

At its core, Polished is designed as an *oskit-style* library: a modular Rust foundation for building applications or system software that can be reused or replaced with ease. **Polished OS** serves as both a demonstration and testbed for this library, showcasing its capabilities in constructing a full operating system while adhering to modern Rust idioms for safety, performance, and maintainability.

______________________________________________________________________

## Why a Monorepo with Cargo Workspace?

Polished OS uses a monorepo structure with a Cargo workspace instead of separate repositories for each crate. This approach was chosen for several reasons:

- **Streamlined Development:** Many changes require updates across multiple crates. Having all components in a single repository makes it easy to coordinate and test these changes together, reducing friction and context switching.
- **Consistent Tooling:** Cargo workspaces provide unified commands for building, testing, and managing dependencies across all crates, simplifying the development workflow.
- **Atomic Commits:** Changes that span multiple crates can be committed and reviewed together, ensuring consistency and reducing the risk of breaking the build.
- **Simplified Dependency Management:** Shared dependencies and versions are managed centrally, avoiding duplication and version drift between crates.
- **Easier Refactoring:** Refactoring APIs or internal interfaces is much simpler when all affected code is in one place.

This structure is common in Rust systems projects and is especially helpful for early-stage OS development, where rapid iteration and cross-crate changes are frequent.

______________________________________________________________________

## Project Structure

This repository is organized as a Cargo workspace. Each component plays a role in building and running Polished OS:

- **bootloader/** — UEFI bootloader written in Rust using `uefi-rs`, responsible for loading the kernel and passing control to it.
- **kernel/** — Core of the OS. Handles memory management, initialization, and (eventually) POSIX/libc compatibility.
- **elf_loader/** — Loads ELF binaries, used by the bootloader to load the kernel.
- **serial_logging/** — Serial port output for debugging and diagnostics.
- **graphics/** — Basic framebuffer drawing and graphics primitives.
- **interrupts/** — Interrupt Descriptor Table (IDT) setup and handling.
- **gdt/** — Global Descriptor Table initialization and management.
- **ps2/** — PS/2 controller support for keyboard and mouse.
- **scancodes/** — Keyboard scancode translation and processing.
- **memory/** — Memory operations (e.g., `memset`, `memcpy`, etc.).
- **files/** — (Planned) Filesystem and storage abstraction. Currently used only for kernel loading.
- **panic_handler/** — Custom panic handler for kernel runtime.
- **x86_commands/** — Low-level x86 instructions and utilities.

______________________________________________________________________

## Features

- UEFI bootloader (via `uefi-rs`)
- ELF kernel loading
- Custom heap allocator (buddy system)
- Serial output logging
- Modular, scalable workspace
- Early memory operation support

______________________________________________________________________

## Building and Running

### Prerequisites

- Rust (nightly toolchain)
- QEMU (for emulation)
- OVMF firmware (from EDK2, for UEFI boot)
- `mtools` and `xorriso` (for image creation)

### Quick Start

Build and run Polished OS in QEMU:

```sh
make run
```

To build and run in release mode (optimized binaries):

```sh
RELEASE=1 make run
```

### Makefile Targets

The following `make` targets are available for building, running, and debugging Polished OS:

| Target | Description |
|--------------------------|-----------------------------------------------------|
| `make run` | Build everything and run in QEMU (default) |
| `make check-artifacts` | Build the kernel and bootloader only |
| `make fat` | Create a FAT EFI system partition image |
| `make iso` | Create a bootable ISO image |
| `make qemu` | Run the built ISO in QEMU (graphical) |
| `make qemu-nographic` | Run QEMU in headless (no graphics) mode |
| `make qemu-gdb` | Run QEMU with GDB stub (graphical) |
| `make qemu-gdb-nographic`| Run QEMU with GDB stub, no graphics |
| `make qemu-debug` | Run QEMU with extra debug output (interrupts) |
| `make qemu-debug-nographic`| QEMU debug output, no graphics |
| `make rust-clean` | Clean Rust build artifacts |
| `make clean` | Clean all build artifacts and images |

For most workflows, `make run` is sufficient. For advanced debugging or headless operation, use the appropriate `qemu-*` targets.

### Cleaning Build Artifacts

```sh
make clean
```

______________________________________________________________________

## Roadmap

- [x] UEFI bootloader
- [x] ELF kernel loading
- [x] Custom heap allocator
- [x] Serial logging
- [ ] Basic filesystem support (beyond kernel loading)
- [ ] libc and POSIX compatibility layers
- [ ] Userland process support (process management, syscalls)
- [ ] Networking stack (TCP/IP, sockets)
- [ ] Graphical interface (windowing, input, basic compositor)
- [ ] Expand device driver support (storage, USB, networking, etc.)
- [ ] Security features (permissions, user authentication)
- [ ] Power management and resource accounting
- [ ] Debugging, tracing, and logging facilities

______________________________________________________________________

## Oskit/Library Goals

- Modularize and document each subsystem for reusability (memory, interrupts, graphics, files, etc.)
- Provide clear APIs and examples for using each oskit component in other projects
- Add more x86 helpers and abstractions (MSR, port I/O, paging, etc.)
- Improve test coverage and add example/test kernels for oskit consumers

______________________________________________________________________

For more details, see the kernel and interrupts TODO lists. Contributions and suggestions are welcome!

______________________________________________________________________

## Contributing

Contributions are welcome. The project is in an early stage, so feedback, issues, and pull requests are appreciated. Feel free to open an issue to suggest improvements or report bugs.

______________________________________________________________________

## License

Unless otherwise noted, components are licensed under the [zlib License](https://zlib.net/zlib_license.html):

- bootloader
- elf_loader
- files
- gdt
- graphics
- interrupts
- memory
- ps2
- panic_handler
- scancodes
- serial_logging
- x86_commands

The `kernel` and OS integration code are licensed under the **GNU General Public License v3.0 (GPL-3.0)**:

- [kernel](./kernel/)
- Polished OS–specific integration code (to be added)

See the [LICENSE](./LICENSE) file for full details.

______________________________________________________________________

## Acknowledgments

- [uefi-rs](https://github.com/rust-osdev/uefi-rs) — UEFI support in Rust
- [buddy_system_allocator](https://github.com/rcore-os/buddy_system_allocator) — Heap allocator
- Rust OSDev community — For resources, examples, and inspiration

______________________________________________________________________

Polished OS is actively in development. Stay tuned for updates, features, and releases.
