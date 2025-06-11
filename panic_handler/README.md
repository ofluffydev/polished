# polished_panic_handler

A custom panic handler for the Polished OS kernel and other low-level, `no_std` Rust environments.

______________________________________________________________________

## What is this crate?

`polished_panic_handler` provides a robust, serial-port-based panic handler for use in operating system kernels, bootloaders, and other environments where the Rust standard library is unavailable. It is designed for use in the Polished OS project, but can be reused in any similar context.

### Key Features

- **Serial output:** All panic information is sent over the serial port using the `serial_logging` crate, making it accessible even when no display is available.
- **Detailed diagnostics:** Outputs the panic location (file, line, column) and message, if available.
- **Safe halting:** After logging, the CPU is halted to prevent further execution, as continuing after a panic is unsafe in kernel code.
- **Modular:** Can be used as a standalone crate in any `no_std` Rust project.

______________________________________________________________________

## How does Rust use this panic handler?

In standard Rust applications, the standard library provides a default panic handler. However, in `no_std` environments (such as kernels and bootloaders), you must provide your own panic handler. Rust looks for a function marked with the `#[panic_handler]` attribute. When a panic occurs (e.g., via `panic!()` or an assertion failure), this function is called with a reference to a \[`core::panic::PanicInfo`\] struct, which contains information about the panic location and message.

This crate implements such a function, ensuring that all panics are logged and the system is safely halted.

______________________________________________________________________

## How it works

- When a panic occurs, the function marked with `#[panic_handler]` is invoked automatically by the Rust runtime.
- The handler logs a generic error message and detailed panic info (location, message) over the serial port.
- The handler then halts the CPU in an infinite loop, using the `cli` and `hlt` instructions to ensure the system does not continue in an unsafe state.

______________________________________________________________________

## Usage

1. **Add as a dependency:**
   In your kernel or bootloader's `Cargo.toml`:
   ```toml
   polished_panic_handler = { path = "../panic_handler" }
   ```
1. **Link the crate:**
   In your main crate, ensure you link to the panic handler. In most cases, simply depending on it is enough, as the panic handler is registered via the attribute.
1. **Serial logging:**
   Ensure the `serial_logging` crate is initialized early in your boot process so that panic output is visible.

______________________________________________________________________

## Example Output

When a panic occurs, the serial port will output something like:

```
Kernel panic occurred!
=== PANIC ===
Location: src/main.rs:42:13
Message: assertion failed: x == 0
=============
```

______________________________________________________________________

## License

This crate is licensed under the zlib License. See the root LICENSE file for details.

______________________________________________________________________

## See Also

- [serial_logging](../serial_logging/): Serial port output utilities
- [Polished OS](../README.md): The main project using this panic handler
