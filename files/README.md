# Polished Files Library

This crate is part of [Polished OS](../README.md), an experimental operating system written in Rust. The `files` library provides file loading and (eventually) filesystem abstraction for use in UEFI bootloaders, kernels, or other system software where direct file access is required in a `no_std` environment.

______________________________________________________________________

## Overview

The `files` library currently focuses on UEFI file loading, providing a minimal and ergonomic interface for reading files from the UEFI Simple File System protocol. It is designed to be modular, safe, and extensible for future filesystem and storage support.

______________________________________________________________________

## How It Works

### UEFI File Loading

The main entry point is the `read_file` function (enabled with the `uefi` feature):

```rust
let data = read_file("EFI/BOOT/hello.txt")?;
// Use `data` as needed
```

**Steps performed:**

1. Converts the UTF-8 path to a UEFI-compatible UTF-16 string (`CString16`).
1. Obtains the UEFI Simple File System protocol for the current image using UEFI boot services.
1. Wraps the protocol in a `FileSystem` abstraction (from the `uefi` crate).
1. Reads the file contents into a heap-allocated buffer (`Vec<u8>`), returning the data or an error.

This approach allows safe and convenient file loading in `no_std` UEFI environments, such as bootloaders or early kernel code.

______________________________________________________________________

## Features

- UEFI file loading via the Simple File System protocol (with `uefi` feature)
- Modular, `no_std`-compatible design
- Safe Rust abstractions for file access
- Designed for use in OS bootloaders and kernel environments

______________________________________________________________________

## Usage

Add this crate as a dependency in your Cargo workspace. If you are building for UEFI, enable the `uefi` feature:

```toml
[dependencies]
polished_files = { path = "../files", features = ["uefi"] }
```

______________________________________________________________________

## License

Unless otherwise noted, all code in this crate is licensed under the [zlib License](https://zlib.net/zlib_license.html):

> This software is provided 'as-is', without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.
>
> Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:
>
> 1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.
> 1. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.
> 1. This notice may not be removed or altered from any source distribution.

See the [LICENSE](../LICENSE) file for full details.

______________________________________________________________________

## Acknowledgments

- [uefi-rs](https://github.com/rust-osdev/uefi-rs) — UEFI support in Rust
- Rust OSDev community — For resources, examples, and inspiration

______________________________________________________________________

This library is actively developed as part of Polished OS. Feedback and contributions are welcome!
