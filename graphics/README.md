# Polished Graphics Library

This crate is part of [Polished OS](../README.md), an experimental operating system written in Rust. The graphics library provides low-level graphics and framebuffer routines for use in OS kernels, bootloaders, or other system software where direct framebuffer access is required.

______________________________________________________________________

## Overview

The graphics library exposes two main modules:

- **framebuffer**: Initialization and management of the framebuffer, including structures describing its memory layout, pixel format, and display properties.
- **drawing**: Basic drawing routines, such as line drawing (using Bresenham's algorithm) and demo patterns, that operate directly on the framebuffer.

The library is written for `no_std` environments and is intended to be portable across different platforms, with special support for UEFI environments via the `uefi` feature flag.

______________________________________________________________________

## How It Works

### Framebuffer Management

A framebuffer is a contiguous region of memory mapped to the display hardware. Each pixel is represented by a value (typically 32 bits for RGBA), and the display hardware reads this memory to show the image on the screen. By writing to the framebuffer, software can directly control what appears on the display.

The `framebuffer` module provides:

- `FramebufferInfo`: A struct describing the framebuffer's address, size, width, height, stride (pixels per row), and pixel format.
- UEFI-specific initialization (with the `uefi` feature): Uses the UEFI Graphics Output Protocol (GOP) to discover and initialize the framebuffer at boot time.

### Drawing Routines

The `drawing` module provides basic drawing functions, including:

- `draw_bresenham`: Draws a line between two points using Bresenham's algorithm, writing directly to the framebuffer memory.
- `framebuffer_x_demo`: Draws an 'X' across the entire framebuffer as a demonstration.

All drawing routines operate by calculating the correct memory offset for each pixel and writing color values (currently hardcoded to white for demos) directly to the framebuffer.

______________________________________________________________________

## Features

- UEFI framebuffer initialization (via `uefi-rs`)
- Modular, `no_std`-compatible design
- Basic drawing primitives (lines, demo patterns)
- Safe Rust abstractions for framebuffer access

______________________________________________________________________

## Usage

Add this crate as a dependency in your Cargo workspace. If you are building for UEFI, enable the `uefi` feature:

```toml
[dependencies]
polished_graphics = { path = "../graphics", features = ["uefi"] }
```

In your code:

```rust
use polished_graphics::framebuffer::{FramebufferInfo};
use polished_graphics::drawing::{draw_bresenham, framebuffer_x_demo};

// For UEFI environments:
#[cfg(feature = "uefi")]
let mut fb = polished_graphics::initialize_framebuffer();

// Draw an X across the screen
framebuffer_x_demo(&mut fb);
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
