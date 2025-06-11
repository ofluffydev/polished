//! # x86_commands: Architecture-Specific Low-Level x86_64 Operations
//!
//! This crate provides direct access to low-level x86/x86_64 hardware commands, intended for use in OS kernels, bootloaders, or other bare-metal environments. All functions are highly architecture-specific and use inline assembly to interact with hardware directly. These commands are not portable and will only work on x86-family CPUs.
//!
//! ## Architecture-Specific Notes
//!
//! - All functions in this crate assume an x86 or x86_64 environment. They use I/O port instructions (`in`, `out`) and other CPU-specific features that are not available on other architectures (such as ARM, RISC-V, etc).
//! - Attempting to use these functions on non-x86 hardware will result in a compile-time or runtime error.
//! - These routines are typically only safe to use in kernel or bootloader code, not in userspace or general application code.
//!
//! ## Example Usage
//!
//! ```rust
//! use x86_commands::disable_pic;
//! disable_pic();
//! ```
//!
//! This will mask all interrupts from the legacy Programmable Interrupt Controller (PIC), which is a common step in modern x86_64 kernels that use the APIC instead.

#![no_std]

use core::arch::asm;

/// Disables the legacy Programmable Interrupt Controller (PIC) on x86/x86_64 systems.
///
/// # Architecture
/// This function is specific to x86-family CPUs. It uses the `out` instruction to write to the PIC's I/O ports (0x21 for the master PIC, 0xA1 for the slave PIC).
///
/// - On modern x86_64 systems, the legacy PIC is often replaced by the APIC, but the PIC must still be masked to prevent spurious interrupts.
/// - This function is a no-op on non-x86 architectures and will not compile there.
///
/// # Safety
/// This function uses inline assembly to directly access hardware ports. It should only be called in a privileged (kernel or bootloader) context.
///
/// # Example
/// ```rust
/// x86_commands::disable_pic();
/// ```
pub fn disable_pic() {
    unsafe {
        // Mask all interrupts on both PICs by writing 0xFF to their data ports.
        // 0x21: Master PIC data port
        // 0xA1: Slave PIC data port
        // This disables all IRQs from the legacy PIC, which is required before enabling the APIC.
        asm!(
            "mov al, 0xFF", // Set AL register to 0xFF (all bits set)
            "out 0xA1, al", // Write AL to slave PIC data port (0xA1)
            "out 0x21, al", // Write AL to master PIC data port (0x21)
            options(nostack, nomem, preserves_flags)
        );
    }
}
