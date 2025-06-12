# Project Polished

![Project Polished Banner](./polished-banner.png)

Photo by <a href="https://unsplash.com/@lifelivedinmono?utm_content=creditCopyText&utm_medium=referral&utm_source=unsplash">Gordon Gerard McLean</a> on <a href="https://unsplash.com/photos/a-window-with-rain-drops-on-it-ual-ZkL2IXQ?utm_content=creditCopyText&utm_medium=referral&utm_source=unsplash">Unsplash</a> (With modifications)

**Project Polished** is an experimental operating system foundation built to be accessible, modern, and more than just a personal project. It aims to eventually support `libc` and POSIX standards, making it suitable for general-purpose use.

---

## Why Project Polished? A Welcoming Alternative to Elitism in OSDev

One of the core purposes of project polished is to provide a welcoming, inclusive, and modern resource for operating system development—especially for those learning or experimenting with OSDev for the first time. Unlike the well-known https://wiki.osdev.org/, which unfortunately contains discouraging and elitist messages (such as the "Beginner mistakes" section that claims you need 10 years of experience, or that you are likely experiencing the Dunning-Kruger effect), project polished believes everyone learns at their own pace and should never be scared off by gatekeeping or arrogance. We reject the notion that only "experts" can build an OS, and we believe that teaching and encouragement are far more valuable than discouragement and exclusion.

Project polished is designed to teach, not gatekeep. We also address the outdated and C-centric focus of much of the existing OSDev material: project polished is written in safe, efficient Rust, and aims to provide a clean, modern, and reusable library (oskit-style) for Rust OS development. While we are not "anti-C," we believe Rust offers a better foundation for safety and maintainability in new projects.

We welcome learners, tinkerers, and experts alike—everyone is encouraged to experiment, contribute, and grow.

---

## Project Polished vs. Polished OS

**Project Polished** is the core OS kit and library ecosystem—a modular, reusable foundation for building operating systems and system software in Rust. It provides safe, modern, and efficient building blocks for OS development, designed to be used as a toolkit or library in your own projects.

**Polished OS** is a demonstration (but fully usable) operating system built entirely using components from the Project Polished ecosystem. It serves as both a reference implementation and a testbed, showcasing how the OS kit can be used to construct a real, working operating system. Polished OS only uses code and libraries from Project Polished, ensuring a clean, self-contained example for others to follow or build upon.

---

## Why a Monorepo with Cargo Workspace?

Project polished uses a monorepo structure with a Cargo workspace instead of separate repositories for each crate. This approach was chosen for several reasons:

- **Streamlined Development:** Many changes require updates across multiple crates. Having all components in a single repository makes it easy to coordinate and test these changes together, reducing friction and context switching.
- **Consistent Tooling:** Cargo workspaces provide unified commands for building, testing, and managing dependencies across all crates, simplifying the development workflow.
- **Atomic Commits:** Changes that span multiple crates can be committed and reviewed together, ensuring consistency and reducing the risk of breaking the build.
- **Simplified Dependency Management:** Shared dependencies and versions are managed centrally, avoiding duplication and version drift between crates.
- **Easier Refactoring:** Refactoring APIs or internal interfaces is much simpler when all affected code is in one place.

This structure is common in Rust systems projects and is especially helpful for early-stage OS development, where rapid iteration and cross-crate changes are frequent.

---

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

---

## Features

- UEFI bootloader (via `uefi-rs`)
- ELF kernel loading
- Custom heap allocator (buddy system)
- Serial output logging
- Modular, scalable workspace
- Early memory operation support

---

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

| Target                      | Description                                   |
| --------------------------- | --------------------------------------------- |
| `make run`                  | Build everything and run in QEMU (default)    |
| `make check-artifacts`      | Build the kernel and bootloader only          |
| `make fat`                  | Create a FAT EFI system partition image       |
| `make iso`                  | Create a bootable ISO image                   |
| `make qemu`                 | Run the built ISO in QEMU (graphical)         |
| `make qemu-nographic`       | Run QEMU in headless (no graphics) mode       |
| `make qemu-gdb`             | Run QEMU with GDB stub (graphical)            |
| `make qemu-gdb-nographic`   | Run QEMU with GDB stub, no graphics           |
| `make qemu-debug`           | Run QEMU with extra debug output (interrupts) |
| `make qemu-debug-nographic` | QEMU debug output, no graphics                |
| `make rust-clean`           | Clean Rust build artifacts                    |
| `make clean`                | Clean all build artifacts and images          |

For most workflows, `make run` is sufficient. For advanced debugging or headless operation, use the appropriate `qemu-*` targets.

### Cleaning Build Artifacts

```sh
make clean
```

---

## Roadmap

- [x] UEFI bootloader
- [x] ELF kernel loading
- [x] Custom heap allocator
- [x] Serial logging
- [ ] VirtIO block device support (requires PCI enumerator and VirtIO block driver)
  - [ ] PCI device enumeration (detect VirtIO PCI devices, vendor ID 0x1AF4)
  - [ ] Map PCI BARs for MMIO register access
  - [ ] Implement Virtqueue structures for I/O
  - [ ] VirtIO block driver (spec v1.1+)
  - [ ] Documentation: What is VirtIO? (Paravirtualized device interface for efficient I/O)
- [ ] Complex filesystem support
  - [ ] ext2 filesystem driver
  - [ ] ext3 filesystem driver
  - [ ] ext4 filesystem driver
  - [ ] Support for additional filesystems (FAT, ISO9660, etc.)
  - [ ] Experimental Btrfs support (long-term goal)
  - [ ] Unified VFS layer and mount system
- [ ] libc and POSIX compatibility
  - [ ] Implement libc and POSIX syscalls with a focus on safety and optimization by default
  - [ ] Provide a modern, safe, and efficient system call interface
  - [ ] Compatibility layer for existing POSIX software
- [ ] Userland process support
  - [ ] Process creation and management (fork/exec)
  - [ ] User/kernel privilege separation
  - [ ] Inter-process communication (IPC)
  - [ ] Signals and process control
- [ ] Networking
  - [ ] Full wired (Ethernet) network stack
  - [ ] Full WiFi (802.11) network stack
  - [ ] TCP/IP, UDP, and socket API
  - [ ] DHCP, DNS, and common network protocols
- [ ] Graphical interface
  - [ ] Compositor for window management and rendering
  - [ ] Basic windowing system and GUI toolkit
  - [ ] Hardware-accelerated graphics (future)

---

## Contributing

Contributions are welcome. The project is in an early stage, so feedback, issues, and pull requests are appreciated. Feel free to open an issue to suggest improvements or report bugs.

---

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

---

## Acknowledgments

- [uefi-rs](https://github.com/rust-osdev/uefi-rs) — UEFI support in Rust
- [buddy_system_allocator](https://github.com/rcore-os/buddy_system_allocator) — Heap allocator
- Rust OSDev community — For resources, examples, and inspiration

---

Project Polished is actively in development. Stay tuned for updates, features, and releases.
