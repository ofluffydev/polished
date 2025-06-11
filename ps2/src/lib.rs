//! PS/2 Controller Initialization Library
//!
//! This library provides low-level routines for initializing the PS/2 controller and keyboard on x86 systems.
//! It is intended for use in OS kernels or bootloaders where direct hardware access is required.
//!
//! # Features
//! - Remaps the Programmable Interrupt Controller (PIC) to avoid conflicts with CPU exceptions.
//! - Configures the PS/2 controller and keyboard device, including IRQ unmasking and device enabling.
//! - Provides safe wrappers for port I/O using inline assembly.
//! - Logs initialization steps using the `serial_logging` crate.
//!
//! # Safety
//! All hardware access is performed in `unsafe` blocks. Use with caution and only in appropriate contexts (e.g., kernel mode).

#![no_std]

extern crate alloc;

// PS/2 controller initialization for keyboard (and optionally mouse)
use alloc::format;
use polished_serial_logging::info;

/// Write a byte to an I/O port using the `out` instruction.
///
/// # Safety
/// This function performs raw hardware access and is unsafe.
///
/// # Arguments
/// * `port` - The I/O port to write to.
/// * `val` - The byte value to write.
///
/// # Inline Assembly
/// Uses the `out dx, al` instruction to send `val` to `port`.
#[inline]
unsafe fn outb(port: u16, val: u8) {
    // Inline assembly: out dx, al
    // Sends the value in AL to the port in DX
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Read a byte from an I/O port using the `in` instruction.
///
/// # Safety
/// This function performs raw hardware access and is unsafe.
///
/// # Arguments
/// * `port` - The I/O port to read from.
///
/// # Returns
/// The byte read from the port.
///
/// # Inline Assembly
/// Uses the `in al, dx` instruction to read a byte from `port` into `val`.
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") val,
            options(nomem, nostack, preserves_flags)
        );
    }
    val
}

/// Initialize the PS/2 controller and keyboard device.
///
/// This function performs the following steps:
/// 1. Remaps the PIC to avoid conflicts with CPU exceptions.
/// 2. Unmasks the keyboard IRQ and masks all slave IRQs.
/// 3. Flushes the PS/2 controller output buffer.
/// 4. Disables both keyboard and mouse devices.
/// 5. Configures the PS/2 controller to enable keyboard IRQ and disable mouse IRQ.
/// 6. Enables the keyboard device and resets it.
/// 7. Enables keyboard scanning.
/// 8. Logs all major steps and responses for debugging.
///
/// # Safety
/// This function must be called in a context where direct hardware access is permitted (e.g., kernel mode).
#[allow(clippy::uninlined_format_args)]
pub fn ps2_init() {
    info("Initializing PS/2 controller...");
    unsafe {
        // --- PIC Remapping ---
        // The PIC (Programmable Interrupt Controller) is remapped so that its IRQs do not overlap with CPU exceptions (0x00-0x1F).
        // Master PIC is mapped to 0x20-0x27, Slave PIC to 0x28-0x2F.
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, 0x20); // Master offset 0x20
        outb(0xA1, 0x28); // Slave offset 0x28
        outb(0x21, 0x04); // Tell Master about Slave at IRQ2
        outb(0xA1, 0x02); // Tell Slave its cascade identity
        outb(0x21, 0x01); // 8086 mode
        outb(0xA1, 0x01); // 8086 mode
        // Unmask IRQ1 (keyboard) and IRQ2 (cascade) at master PIC, mask all slave IRQs
        let master_mask = inb(0x21);
        let new_master = master_mask & !((1 << 1) | (1 << 2)); // unmask IRQ1 & IRQ2
        outb(0x21, new_master);
        outb(0xA1, 0xFF); // mask all slave interrupts
        // Read port 0x60 once to clear any stale scancode after remap
        let _ = inb(0x60);

        // --- Helper Closures for Buffer Status ---
        // Wait for input buffer to be clear (ready to accept commands)
        let wait_input_clear = || {
            for _ in 0..10000 {
                let status: u8;
                core::arch::asm!(
                    "in al, dx",
                    in("dx") 0x64u16,
                    out("al") status,
                    options(nomem, nostack, preserves_flags)
                );
                if status & 0x02 == 0 {
                    break;
                }
            }
        };
        // Wait for output buffer to be set (data available to read)
        let wait_output_set = || {
            for _ in 0..10000 {
                let status: u8;
                core::arch::asm!(
                    "in al, dx",
                    in("dx") 0x64u16,
                    out("al") status,
                    options(nomem, nostack, preserves_flags)
                );
                if status & 0x01 != 0 {
                    break;
                }
            }
        };

        // --- Flush Output Buffer ---
        wait_output_set();
        let mut _dummy: u8 = 0;
        core::arch::asm!(
            "in al, dx",
            in("dx") 0x60u16,
            out("al") _dummy,
            options(nomem, nostack, preserves_flags)
        );

        // --- Disable Devices ---
        // Disable keyboard
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xAD", // disable keyboard
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        // Disable mouse
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xA7", // disable mouse
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );

        // --- Set Controller Configuration Byte ---
        // Enable keyboard IRQ, disable mouse IRQ, no translation
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0x20",
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        wait_output_set();
        let mut _config: u8 = 0;
        core::arch::asm!(
            "in al, dx",
            in("dx") 0x60u16,
            out("al") _config,
            options(nomem, nostack, preserves_flags)
        );
        // Set: enable keyboard IRQ (bit 0), disable mouse IRQ (bit 1), clear translation (bit 6)
        _config = (_config | 0x01 | 0x40) & !0x02;
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0x60",
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        wait_input_clear();
        core::arch::asm!(
            "out 0x60, al",
            in("al") _config,
            options(nomem, nostack, preserves_flags)
        );

        // --- Enable Keyboard Device ---
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xAE",
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );

        // --- Keyboard Reset and Enable Scanning ---
        // Send reset command (0xFF) to keyboard
        wait_input_clear();
        outb(0x60, 0xFF);
        wait_output_set();
        let ack = inb(0x60);
        let msg = format!("Keyboard RESET ACK: {:#x}", ack);
        info(&msg);
        if ack == 0xFA {
            // If ACK received, read BAT (Basic Assurance Test) response
            wait_output_set();
            let bat = inb(0x60);
            let msg = format!("Keyboard BAT response: {:#x}", bat);
            info(&msg);
        } else {
            let msg = format!("Keyboard did not ACK reset as expected: {:#x}", ack);
            info(&msg);
        }
        // Enable keyboard scanning (0xF4)
        wait_input_clear();
        outb(0x60, 0xF4);
        wait_output_set();
        let scan_ack = inb(0x60);
        let msg = format!("Keyboard scanning ACK: {:#x}", scan_ack);
        info(&msg);
        // Unmask IRQ1 (keyboard) again after all initialization
        let master_mask = inb(0x21);
        let new_master = master_mask & !(1 << 1);
        outb(0x21, new_master);
    }
    info("PS/2 controller initialized");
}
