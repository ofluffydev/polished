# PCI Bus Access and Enumeration Library (`polished_pci`)

> **Note:** This is an early version of the `polished_pci` crate. The long-term goal is to provide a drop-in, robust PCI access and enumeration solution for x86 kernel and OS development in Rust.

This crate provides low-level routines for accessing and enumerating PCI devices on x86 systems. It is designed for use in operating system kernels or bootloaders written in Rust, where direct hardware access is required. `polished_pci` is a core component of the [Polished OS](../README.md) project, enabling device discovery and initialization for modern and legacy hardware.

______________________________________________________________________

## Overview

The `polished_pci` library offers safe(ish) Rust abstractions over the raw I/O port operations needed to:

- Read from and write to PCI configuration space
- Enumerate devices on PCI bus 0
- Print device information using the `serial_logging` crate
- Provide safe wrappers for port I/O using inline assembly

All hardware access is performed in `unsafe` blocks, as is required for direct port I/O on x86 hardware.

For current features and todo items, see the [TODO.md](TODO.md) file.

______________________________________________________________________

## Features

- **PCI Configuration Space Access:** Read 32-bit values from PCI configuration registers using standard I/O ports (0xCF8/0xCFC).
- **Device Enumeration:** Scan all 32 possible devices on PCI bus 0, detect present devices, and extract vendor/device/class information.
- **Device Information Logging:** Print human-readable information about each detected device using the `serial_logging` crate (or a no-op logger if disabled).
- **Vendor/Class Decoding:** Recognize common vendor IDs (Intel, NVIDIA, QEMU, VirtIO) and class codes for easier debugging.
- **No-Std Compatible:** Designed for use in `no_std` environments such as kernels and bootloaders.

______________________________________________________________________

## How to Use in Rust

### 1. Add as a Dependency

If using as part of a workspace (as in Polished OS), add to your `Cargo.toml`:

```toml
[dependencies]
polished_pci = { path = "../pci" }
```

Or, if using features:

```toml
[features]
default = ["polished_serial_logging"]
polished_serial_logging = ["dep:polished_serial_logging"]

[dependencies]
polished_serial_logging = { path = "../serial_logging", optional = true }
```

### 2. PCI Enumeration Demo

Call the `pci_enumeration_demo()` function early in your kernel or bootloader initialization sequence, after setting up basic memory and logging:

```rust
polished_pci::pci_enumeration_demo();
```

This will:

- Scan PCI bus 0 for devices
- Print vendor, device, class, and subclass information for each device found
- Log all output to the serial port (if enabled)

### 3. Safety

All functions that perform hardware access are marked `unsafe` internally. The public API (`pci_enumeration_demo`) is safe to call, but must only be used in a context where direct hardware access is permitted (i.e., kernel mode, not in userland or standard OS processes).

______________________________________________________________________

## Example: Kernel Integration

```rust
// In your kernel's main.rs or initialization code:
extern crate polished_pci;

fn main() {
    // ...other early setup...
    polished_pci::pci_enumeration_demo();
    // ...continue kernel boot...
}
```

______________________________________________________________________

## Implementation Details

- **Port I/O:** Uses inline assembly (`core::arch::asm!`) for `inl` and `outl` operations to access PCI configuration space.
- **PCI Addressing:** Constructs configuration addresses according to the PCI specification for bus, device, function, and register offset.
- **Device Detection:** Checks for valid vendor IDs (0xFFFF means no device present).
- **Logging:** Uses the `serial_logging` crate to log device information, or a no-op logger if the feature is disabled.
- **No-Std:** Uses `#![no_std]` and the `alloc` crate for string formatting.

______________________________________________________________________

## How PCI Device Enumeration Works

The PCI (Peripheral Component Interconnect) bus is a standard for connecting peripheral devices to a computer's processor and memory. Device enumeration is the process of scanning the bus to discover all connected devices and their capabilities. Here’s how it works in the context of OS development:

- **I/O Ports:** PCI configuration space is accessed via I/O ports 0xCF8 (address) and 0xCFC (data).
- **Address Construction:** The OS writes a 32-bit address to 0xCF8, specifying the bus, device, function, and register offset.
- **Data Access:** The OS reads or writes 32-bit values at 0xCFC to access device configuration registers.
- **Device Detection:** If the vendor ID read from a device is 0xFFFF, no device is present at that slot.
- **Class Codes:** Devices are classified by class and subclass codes, which indicate their function (e.g., network, storage, display).

### Typical Enumeration Steps

1. **Scan Devices:** Loop over all possible device numbers (0-31) on bus 0.
1. **Read Vendor ID:** If the vendor ID is not 0xFFFF, a device is present.
1. **Read Device Info:** Extract device ID, class code, and subclass for each present device.
1. **Log Information:** Print or log the details for debugging and driver initialization.

______________________________________________________________________

## PCI and Polished OS

`polished_pci` is a foundational crate for Polished OS, enabling the kernel to:

- Detect and identify hardware devices (including VirtIO, QEMU, Intel, and more)
- Support device drivers for storage, networking, and graphics
- Enable advanced features like VirtIO block devices and network interfaces
- Provide robust hardware abstraction for future expansion

PCI enumeration is a critical step in modern OS initialization, as it allows the kernel to discover and initialize all hardware required for boot and runtime operation.

______________________________________________________________________

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root LICENSE file for details.

______________________________________________________________________

## References & Acknowledgments

- [OSDev.org PCI](https://wiki.osdev.org/PCI)
- [PCI Local Bus Specification](https://pcisig.com/specifications)
- [Rust OSDev Community](https://osdev.rs/)

______________________________________________________________________

For questions or contributions, see the main Polished OS repository.
