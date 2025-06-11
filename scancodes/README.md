# Scancodes Library (`scancodes`)

This crate provides utilities for handling keyboard scancodes in x86 OS development. It is designed for use in kernels or low-level system software that needs to interpret raw keyboard input from hardware interrupts.

______________________________________________________________________

## Overview

When a key is pressed or released on a PC keyboard, the hardware sends a *scancode* to the system via the PS/2 controller. These scancodes are received by the OS through keyboard interrupts (typically IRQ1). The `scancodes` library provides:

- Constants and tables for common keyboard scancode sets (e.g., Set 1, used by most PC keyboards).
- Functions to translate raw scancodes into key events (press/release) and optionally ASCII characters.
- Utilities to help OS kernels decode and process keyboard input in interrupt handlers.

______________________________________________________________________

## Why is this Needed in OS Development?

- **Raw Input:** The hardware delivers only raw scancodes, not ASCII or Unicode. The OS must interpret these codes.
- **Interrupt Context:** Keyboard input is received in the interrupt handler for IRQ1. The handler must quickly decode the scancode and queue or process the event.
- **Multiple Sets:** Keyboards may use different scancode sets (though Set 1 is most common on PCs).
- **Modifiers:** Handling Shift, Ctrl, Alt, and other modifiers requires tracking key state across interrupts.

______________________________________________________________________

## Features

- **Scancode Constants:** Definitions for all standard Set 1 scancodes (make and break codes).
- **Translation Functions:** Convert scancodes to key events and optionally to ASCII (for US QWERTY layout).
- **Modifier State Tracking:** Utilities to help track Shift, Ctrl, Alt, and Caps Lock state.
- **No-Std:** Suitable for use in `#![no_std]` environments (OS kernels, bootloaders).

______________________________________________________________________

## Example Usage in an OS Kernel

### 1. Add as a Dependency

If using as part of a workspace:

```toml
[dependencies]
scancodes = { path = "../scancodes" }
```

### 2. Use in Keyboard Interrupt Handler

```rust
// In your keyboard interrupt handler (IRQ1):
use scancodes::{decode_scancode, KeyEvent};

fn keyboard_interrupt_handler() {
    let scancode = unsafe { inb(0x60) }; // Read from PS/2 data port
    if let Some(event) = decode_scancode(scancode) {
        match event {
            KeyEvent::Pressed(key) => { /* handle key press */ },
            KeyEvent::Released(key) => { /* handle key release */ },
        }
    }
}
```

- `decode_scancode` is a typical function provided to convert a raw scancode into a high-level event.
- Modifier keys (Shift, Ctrl, etc.) can be tracked using helper functions or state machines provided by the library.

### 3. Mapping to ASCII (Optional)

If you want to convert key events to ASCII (for text input):

```rust
if let Some(ascii) = scancodes::key_to_ascii(key, shift_active) {
    // Use ASCII character (e.g., print to screen or buffer)
}
```

______________________________________________________________________

## Implementation Details

- **No-Std:** The library is designed for use in environments without the Rust standard library.
- **Tables:** Uses static tables for fast lookup of scancode-to-key and key-to-ASCII mappings.
- **Extensible:** Can be extended to support other layouts or scancode sets if needed.

______________________________________________________________________

## When to Use This Library

- Writing a custom OS kernel or bootloader in Rust for x86 hardware.
- Need to handle keyboard input at the hardware/interrupt level.
- Require translation from raw scancodes to key events or ASCII.

______________________________________________________________________

## License

This crate is licensed under the [zlib License](https://zlib.net/zlib_license.html). See the root LICENSE file for details.

______________________________________________________________________

## References & Acknowledgments

- [OSDev.org Keyboard](https://wiki.osdev.org/Keyboard)
- [PC Keyboard Scancodes](https://www.win.tue.nl/~aeb/linux/kbd/scancodes-1.html)
- Rust OSDev community

______________________________________________________________________

For questions or contributions, see the main Polished OS repository.
