# PS/2 Controller Initialization Library (`ps2`)

This crate provides low-level routines for initializing and managing the PS/2 controller and keyboard on x86 systems. It is designed for use in operating system kernels or bootloaders written in Rust, where direct hardware access is required.

______________________________________________________________________

## Overview

The `ps2` library offers safe(ish) Rust abstractions over the raw I/O port operations needed to:

- Remap the Programmable Interrupt Controller (PIC) to avoid conflicts with CPU exceptions.
- Configure the PS/2 controller and keyboard device, including IRQ unmasking and device enabling.
- Provide safe wrappers for port I/O using inline assembly.
- Log initialization steps using the `serial_logging` crate.

All hardware access is performed in `unsafe` blocks, as is required for direct port I/O on x86 hardware.

______________________________________________________________________

## Features

- **PIC Remapping:** Ensures hardware interrupts do not overlap with CPU exceptions by remapping the master and slave PICs.
- **PS/2 Controller Setup:** Disables devices, configures the controller, and enables the keyboard device.
- **Keyboard Initialization:** Sends reset and enable commands to the keyboard, verifies responses, and enables keyboard scanning.
- **IRQ Masking:** Unmasks only the required IRQs for keyboard operation, masking all others for safety.
- **Logging:** Uses the `serial_logging` crate to log each major step and hardware response for debugging.
- **Safe Wrappers:** Provides `outb` and `inb` functions for port I/O, wrapped in `unsafe` Rust for explicitness.

______________________________________________________________________

## How to Use in Rust

### 1. Add as a Dependency

If using as part of a workspace (as in Polished OS), add to your `Cargo.toml`:

```toml
[dependencies]
ps2 = { path = "../ps2" }
```

### 2. Initialization

Call the `ps2_init()` function early in your kernel or bootloader initialization sequence, after setting up basic memory and logging:

```rust
ps2::ps2_init();
```

This will:

- Remap the PIC
- Set up the PS/2 controller
- Enable the keyboard
- Log all steps to the serial port

### 3. Safety

All functions that perform hardware access are marked `unsafe` internally. The public API (`ps2_init`) is safe to call, but must only be used in a context where direct hardware access is permitted (i.e., kernel mode, not in userland or standard OS processes).

______________________________________________________________________

## Example: Kernel Integration

```rust
// In your kernel's main.rs or initialization code:
extern crate ps2;

fn main() {
    // ...other early setup...
    ps2::ps2_init();
    // ...continue kernel boot...
}
```

______________________________________________________________________

## Implementation Details

- **Port I/O:** Uses inline assembly (`core::arch::asm!`) for `inb` and `outb` operations.
- **Buffer Status:** Waits for input/output buffer readiness before sending/receiving commands.
- **PIC/IRQ:** Remaps and unmasks only the necessary IRQs for keyboard operation.
- **Keyboard Commands:** Issues reset (`0xFF`) and enable scanning (`0xF4`) commands, and checks for proper acknowledgments.
- **Logging:** All major steps and hardware responses are logged via the `serial_logging` crate for debugging.

______________________________________________________________________

## How the PS/2 Controller Works

The PS/2 controller is a legacy hardware interface used to connect keyboards and mice to x86 PCs. It communicates with devices using a simple serial protocol and is managed by the system firmware and operating system. Here’s how it works in the context of OS development:

- **I/O Ports:** The controller is accessed via specific I/O ports (typically 0x60 for data and 0x64 for commands/status).
- **Interrupts:** When a key is pressed or released, the keyboard sends a scancode to the controller, which then triggers an interrupt (IRQ1) to notify the OS.
- **Polling and Buffering:** The OS can poll the controller’s status registers to check if data is available or if it’s ready to accept new commands.
- **Initialization:** The controller and keyboard must be initialized by the OS after boot. This involves remapping interrupts, enabling/disabling devices, and configuring controller settings.
- **Direct Hardware Access:** All communication is done via port I/O and requires privileged (kernel) mode.

### Typical Initialization Steps

1. **Remap the PIC:** Avoids conflicts between hardware interrupts and CPU exceptions.
1. **Mask/Unmask IRQs:** Ensures only the required interrupts (like the keyboard) are enabled.
1. **Flush Buffers:** Clears any stale data from the controller.
1. **Disable Devices:** Prevents spurious input during setup.
1. **Configure Controller:** Sets IRQ enable bits and other options.
1. **Enable Keyboard:** Powers up the keyboard and enables scanning.

______________________________________________________________________

## PS/2 vs. USB Keyboards: Key Differences

| Feature | PS/2 Keyboard | USB Keyboard |
|------------------------|--------------------------------------|--------------------------------------|
| **Connection** | Legacy 6-pin mini-DIN port | USB port (Type-A, Type-C, etc.) |
| **Communication** | Serial protocol via I/O ports | Packet-based USB protocol |
| **Interrupt Handling** | Hardware IRQ (IRQ1) | Polled or interrupt via USB stack |
| **Initialization** | Direct port I/O, simple commands | Requires USB stack and enumeration |
| **Hotplug Support** | No (must be connected at boot) | Yes (can be plugged/unplugged live) |
| **OS Complexity** | Simple, handled in kernel/firmware | Requires USB host controller driver |
| **Latency** | Very low, direct interrupt | Slightly higher, USB stack overhead |
| **Legacy Support** | Supported in BIOS/UEFI and early OS | May require special drivers in early boot |

### Summary

- **PS/2** is simpler for OS development: you can talk to the hardware directly, handle interrupts, and don’t need a USB stack.
- **USB** keyboards require a full USB host controller driver, device enumeration, and more complex protocol handling, which is much harder to implement in a hobby OS.
- **Hotplugging** is only supported by USB; PS/2 devices must be present at boot.

______________________________________________________________________

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root LICENSE file for details.

______________________________________________________________________

## References & Acknowledgments

- [OSDev.org PS/2 Controller](https://wiki.osdev.org/%228042%22_PS/2_Controller)
- [Rust OSDev Community](https://osdev.rs/)

______________________________________________________________________

For questions or contributions, see the main Polished OS repository.
