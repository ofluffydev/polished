# Polished ELF Loader Library

This crate is part of [Polished OS](../README.md), an experimental operating system written in Rust. The ELF loader provides modular, safe ELF (Executable and Linkable Format) loading functionality for use in UEFI bootloaders or other system software that needs to load and execute ELF binaries, such as kernels or userland applications.

---

## Overview

The `elf_loader` library is responsible for:

- Reading an ELF file from disk (typically from an EFI system partition)
- Parsing the ELF file and iterating over its program headers (segments)
- Allocating memory for each loadable segment at the addresses specified by the ELF headers
- Copying segment data from the file into memory, zero-filling any uninitialized data (BSS)
- Returning the entry point address and a callable function pointer to start the loaded kernel or application

The loader is designed for use in a UEFI environment and leverages UEFI services for memory allocation when the `uefi` feature is enabled.

---

## How It Works

### Loading an ELF File

The main entry point is the `load_kernel` function (enabled with the `uefi` feature):

```rust
let (entry, kernel_entry) = load_kernel("\\EFI\\BOOT\\kernel");
// To start the kernel:
unsafe { kernel_entry() };
```

**Steps performed:**

1. Reads the ELF file from disk into memory using the `polished_files` crate.
2. Parses the ELF file structure using the [`xmas-elf`](https://docs.rs/xmas-elf/) crate.
3. Iterates over each program header (segment):
    - Skips non-loadable segments (e.g., dynamic sections)
    - Allocates memory at the requested virtual address using UEFI services
    - Copies segment data from the file into the allocated memory
    - Zero-fills any remaining memory for uninitialized data (BSS)
4. Returns the entry point address (from the ELF header) and a function pointer to the entry point, which can be called to transfer control to the loaded binary.

---

## Features

- UEFI memory allocation for segment loading (with `uefi` feature)
- Modular, `no_std`-compatible design
- Safe Rust abstractions for ELF parsing and loading
- Designed for use in OS bootloaders and kernel environments

---

## Usage

Add this crate as a dependency in your Cargo workspace. If you are building for UEFI, enable the `uefi` feature:

```toml
[dependencies]
polished_elf_loader = { path = "../elf_loader", features = ["uefi"] }
```

---

## License

Unless otherwise noted, all code in this crate is licensed under the [zlib License](https://zlib.net/zlib_license.html):

> This software is provided 'as-is', without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.
>
> Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:
>
> 1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.
> 2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.
> 3. This notice may not be removed or altered from any source distribution.

See the [LICENSE](../LICENSE) file for full details.

---

## Acknowledgments

- [xmas-elf](https://github.com/philipc/xmas-elf) — ELF parsing in Rust
- [uefi-rs](https://github.com/rust-osdev/uefi-rs) — UEFI support in Rust
- Rust OSDev community — For resources, examples, and inspiration

---

This library is actively developed as part of Polished OS. Feedback and contributions are welcome!
