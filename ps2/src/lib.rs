#![no_std]

extern crate alloc;

// PS/2 controller initialization for keyboard (and optionally mouse)
use alloc::format;
use serial_logging::info;

// Helper functions for port I/O
#[inline]
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags)
        );
    }
}

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

#[allow(clippy::uninlined_format_args)]
pub fn ps2_init() {
    info("Initializing PS/2 controller...");
    unsafe {
        // Remap PIC
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
        // Wait for input buffer to be clear
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
        // Wait for output buffer to be set
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
        // Flush output buffer
        wait_output_set();
        let mut _dummy: u8 = 0;
        core::arch::asm!(
            "in al, dx",
            in("dx") 0x60u16,
            out("al") _dummy,
            options(nomem, nostack, preserves_flags)
        );
        // Disable devices
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xAD", // disable keyboard
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xA7", // disable mouse
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        // Set config byte: enable keyboard IRQ, disable mouse IRQ, no translation
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
        // Enable keyboard device
        wait_input_clear();
        core::arch::asm!(
            "mov al, 0xAE",
            "out 0x64, al",
            options(nomem, nostack, preserves_flags)
        );
        // --- KEYBOARD RESET AND ENABLE SCANNING SEQUENCE ---
        // Reset the keyboard (0xFF)
        wait_input_clear();
        outb(0x60, 0xFF);
        wait_output_set();
        let ack = inb(0x60);
        let msg = format!("Keyboard RESET ACK: {:#x}", ack);
        info(&msg);
        if ack == 0xFA {
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
