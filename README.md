# Polished OS (Project Polished)

Polished OS is a new, experimental operating system designed with the goal of being accessible, modern, and not just a personal project. While still in its early stages, Polished OS aims to eventually support libc and POSIX standards, making it a suitable platform for a wide range of software and users.

## Project Structure

This repository is a Cargo workspace containing all components required to build and run Polished OS:

- **bootloader/**: UEFI bootloader written in Rust, responsible for loading the kernel and passing control to it.
- **kernel/**: The core of Polished OS, written in Rust. Handles memory management, system initialization, and will eventually provide POSIX and libc compatibility.
- **elf_loader/**: Library for loading ELF binaries (used by the bootloader to load the kernel).
- **serial_logging/**: Provides serial port logging for debugging and kernel output.
- **files/**: (Planned, currently only used to load kernel) File system and storage abstractions.

## Features

- UEFI bootloader (Rust, using the `uefi` crate)
- ELF kernel loading
- Custom kernel with heap allocator (buddy system)
- Serial logging for debugging
- Modular workspace structure for easy expansion
- Early support for memory operations (memset, memcpy, etc.)

## Building and Running

### Prerequisites
- Rust nightly toolchain
- QEMU (for emulation)
- EDK2 OVMF firmware (for UEFI)
- `mtools` and `xorriso` (for disk image creation)

### Quick Start

To build and run Polished OS in QEMU, simply run:
```sh
make run
```
This will build the kernel and bootloader, package them into a FAT image, create a bootable ISO, and launch QEMU with the correct firmware and configuration.

To build and run in release mode (for optimized binaries):
```sh
RELEASE=1 make run
```

### Makefile Targets
- `make run` — Build everything and run in QEMU (default target for development)
- `make check-artifacts` — Build the kernel and bootloader
- `make fat` — Create the FAT EFI System Partition image
- `make iso` — Create a bootable ISO image
- `make qemu` — Run the built ISO in QEMU (called by `make run`)
- `make rust-clean` — Clean build artifacts for kernel and bootloader

You can use these targets individually for advanced workflows, but for most users, `make run` is all you need.

### Cleaning
To clean build artifacts:
```sh
make rust-clean
```

## Roadmap
- [x] UEFI bootloader
- [x] ELF kernel loading
- [x] Custom heap allocator
- [x] Serial logging
- [ ] Basic file system support
- [ ] libc and POSIX compatibility
- [ ] Userland process support
- [ ] Networking
- [ ] Graphical interface

## Contributing
Contributions are welcome! The project is in its infancy, so feedback, bug reports, and pull requests are appreciated. Please open an issue or PR if you have suggestions or improvements.

## License
This project is licensed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). See [LICENSE](LICENSE) for details.

## Acknowledgments
- [uefi-rs](https://github.com/rust-osdev/uefi-rs) for UEFI support in Rust
- [buddy_system_allocator](https://github.com/rcore-os/buddy_system_allocator) for heap management
- The Rust OSDev community for inspiration and resources

---

Polished OS is a work in progress. Stay tuned for updates and new features as development continues!
