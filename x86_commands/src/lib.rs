#![no_std]

use core::arch::asm;

/// Disable the PIC
pub fn disable_pic() {
    unsafe {
        // Mask all interrupts on both PICs
        asm!(
            "mov al, 0xFF",
            "out 0xA1, al", // slave PIC
            "out 0x21, al", // master PIC
            options(nostack, nomem, preserves_flags)
        );
    }
}