//! # Hardware Interrupt Handlers
//!
//! This module provides setup routines for hardware interrupt handlers (IRQs), such as the programmable interval timer (PIT) and keyboard controller.
//!
//! ## What are Hardware Interrupts?
//!
//! Hardware interrupts (IRQs) are signals sent by external devices to the CPU, requesting immediate attention. Examples include timer ticks, keyboard presses, and disk I/O completions. The OS must register handlers for these events in the Interrupt Descriptor Table (IDT) to respond appropriately.
//!
//! This module provides a function to register hardware interrupt handlers in the IDT.

use core::arch::asm;

use polished_serial_logging::kprint;
use x86_64::structures::idt::InterruptStackFrame;

pub fn setup_hardware_interrupts(idt: &mut x86_64::structures::idt::InterruptDescriptorTable) {
    idt[32].set_handler_fn(timer_interrupt_handler);
    idt[33].set_handler_fn(keyboard_interrupt_handler);
    idt[44].set_handler_fn(mouse_interrupt_handler);
    idt[46].set_handler_fn(disk_interrupt_handler);
    idt[43].set_handler_fn(network_interrupt_handler);
    idt[55].set_handler_fn(usb_interrupt_handler);
    idt[47].set_handler_fn(other_hardware_interrupt_handler);
    // Add stubs for additional hardware IRQs as listed in README
    // (IRQ 0-15: 32-47, already covered above)
    // If more hardware devices are present, add their handlers here as needed
}

fn send_eoi() {
    unsafe {
        asm!(
            "mov al, 0x20",
            "out 0x20, al",
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // kprint!("[INFO] INT 0x20: Timer interrupt\r\n"); // uncomment this if you want timer to scream at you
    send_eoi();
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let scancode: u8;
    unsafe {
        asm!(
            "in al, 0x60",
            out("al") scancode,
            options(nomem, nostack, preserves_flags)
        );
    }

    if scancode == 0xFA {
        kprint!(
            "[INFO] INT 0x21: Keyboard interrupt, received 0xFA (possible ACK, not a keypress)\r\n"
        );
    } else if scancode & 0x80 == 0 {
        // Only handle key press (make) codes, ignore break codes
        let converted = polished_scancodes::scancode_to_ascii(scancode);
        match converted {
            Some(ascii) if ascii.is_ascii_graphic() || ascii == b' ' => {
                kprint!(
                    "[INFO] INT 0x21: Keyboard interrupt, scancode: {:#x} | ASCII: '{}'\r\n",
                    scancode,
                    ascii as char
                );
            }
            _ => {
                kprint!(
                    "[INFO] INT 0x21: Keyboard interrupt, scancode: {:#x} | ASCII: Unknown\r\n",
                    scancode
                );
            }
        }
    }

    send_eoi();
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x2C: Mouse interrupt\r\n");
    // TODO: Read mouse data, send EOI
}

pub extern "x86-interrupt" fn disk_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x2E: Disk controller interrupt\r\n");
    // TODO: Handle disk I/O, send EOI
}

pub extern "x86-interrupt" fn network_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x2B: Network card interrupt\r\n");
    // TODO: Handle network I/O, send EOI
}

pub extern "x86-interrupt" fn usb_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x37: USB controller interrupt\r\n");
    // TODO: Handle USB I/O, send EOI
}

pub extern "x86-interrupt" fn other_hardware_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x2F: Other hardware device interrupt\r\n");
    // TODO: Handle other hardware, send EOI
}
