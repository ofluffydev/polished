# Serial Logging Library (`serial_logging`)

This crate provides robust serial port logging for x86_64 kernel and bootloader development, with a strong focus on QEMU-based debugging. It enables formatted and raw output to the serial port (COM1, 0x3F8), making it possible to see kernel logs even before graphics or higher-level output is available.

______________________________________________________________________

## Overview

Serial logging is a critical tool for OS developers, especially in early boot stages or when debugging in virtual machines like QEMU. This library offers:

- Macros for formatted serial output (`serial_print!`, `serial_println!`, `serial_log!`, etc.).
- Log level support (info, warning, error, hex output).
- A minimal, dependency-free `kprint!` macro for very early boot or `no_std` contexts.
- Thread-safe output using a spinlock and the `uart_16550` crate.
- Runtime enable/disable of logging.

All output is sent to the first serial port (COM1, 0x3F8), which QEMU can redirect to your terminal or a file for easy debugging.

______________________________________________________________________

## Why Serial Logging?

- **Early Debugging:** Serial output works before graphics or even memory allocators are initialized.
- **QEMU Integration:** QEMU can redirect serial output to your terminal with `-serial stdio`, making it easy to see kernel logs in real time.
- **Minimal Dependencies:** The `kprint!` macro works with only `core::fmt`, making it ideal for the earliest boot stages.
- **Thread Safety:** The main driver uses a spinlock to ensure output is not garbled by concurrent writes.

______________________________________________________________________

## Features

- **Formatted Output:** Use Rust-style formatting macros for serial output.
- **Log Levels:** Macros for info, warning, error, and hex output.
- **Minimal Output:** `kprint!` for direct, dependency-free serial output.
- **Enable/Disable Logging:** Control output at runtime for silent or verbose modes.
- **QEMU-Friendly:** Designed for use with QEMU's `-serial stdio` or `-serial file:...` options.

______________________________________________________________________

## How to Use in OS Development

### 1. Add as a Dependency

If using as part of a workspace:

```toml
[dependencies]
serial_logging = { path = "../serial_logging" }
```

### 2. QEMU Setup

Run QEMU with serial redirection to see logs in your terminal:

```sh
qemu-system-x86_64 -serial stdio -kernel path/to/kernel
```

### 3. Basic Usage in Kernel Code

```rust
use serial_logging::*;

serial_println!("Hello, QEMU serial!");
serial_log!("[INFO] ", "Boot complete");
info("System started");
warn("Low memory");
error!("Failed to load: {}", 42);
```

### 4. Early Boot Output

For output before the main serial driver is initialized, use the minimal macro:

```rust
kprint!("Early boot message: {}", 123);
```

This writes directly to the serial port using inline assembly and does not require the full driver or any heap.

______________________________________________________________________

## Implementation Details

- **Serial Port:** Uses `uart_16550` for main output, and direct port I/O for `kprint!`.
- **Thread Safety:** Uses a spinlock (`spin::Mutex`) to guard the serial port.
- **No-Std:** Fully compatible with `#![no_std]` environments.
- **Macros:** Provide both high-level (formatted, log-level) and low-level (raw) output.
- **Runtime Control:** Logging can be enabled or disabled at runtime.

______________________________________________________________________

## When to Use This Library

- Writing a custom OS kernel or bootloader in Rust for x86_64 hardware.
- Need to debug early boot code or kernel logic in QEMU or other VMs.
- Require output before graphics or higher-level drivers are available.
- Want thread-safe, formatted logging in a `no_std` environment.

______________________________________________________________________

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root LICENSE file for details.

______________________________________________________________________

## References & Acknowledgments

- [OSDev.org Serial Ports](https://wiki.osdev.org/Serial_Ports)
- [uart_16550 crate](https://crates.io/crates/uart_16550)
- [QEMU Serial Redirection](https://wiki.osdev.org/QEMU)

______________________________________________________________________

For questions or contributions, see the main Polished OS repository.
