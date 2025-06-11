//! # Interrupt Handling Library
//!
//! This crate provides low-level interrupt management for x86_64 systems, including the setup of the Interrupt Descriptor Table (IDT), CPU exception handlers, and hardware interrupt handlers. It is designed for use in OS kernels or bootloaders running in a `no_std` environment.
//!
//! ## How Interrupts Work
//!
//! Interrupts are signals that temporarily halt the normal execution flow of the CPU to handle urgent events, such as hardware requests (keyboard, timer, etc.) or software exceptions (division by zero, page faults, etc.). When an interrupt occurs, the CPU saves its current state and jumps to a handler function specified in the Interrupt Descriptor Table (IDT). After the handler completes, the CPU resumes execution where it left off.
//!
//! ### Types of Interrupts
//! - **CPU Exceptions:** Triggered by error conditions in program execution (e.g., invalid opcode, page fault).
//! - **Hardware Interrupts (IRQs):** Triggered by external devices (e.g., keyboard, timer, disk controller).
//! - **Software Interrupts:** Triggered by instructions like `int n` (e.g., for system calls).
//!
//! ## Where Interrupts Are Used
//!
//! - **Exception Handling:** To catch and handle errors such as division by zero or invalid memory access.
//! - **Device Communication:** To respond to hardware events (e.g., keyboard presses, timer ticks).
//! - **Preemptive Multitasking:** To switch between tasks on timer interrupts.
//! - **System Calls:** Some OSes use software interrupts for syscalls, but this library does **not** use the legacy `int 0x80` or similar interrupt-based syscalls.
//!
//! ## System Call Implementation Options
//!
//! There are two main approaches for implementing syscalls on x86_64:
//! 1. **Software Interrupts:** Using instructions like `int 0x80` (legacy, slower, not used here).
//! 2. **Fast Syscall Instructions:** Using `syscall`/`sysret` (preferred for modern OSes, not implemented in this library).
//!
//! This library focuses on exception and hardware interrupt handling, not syscall dispatch.
//!
//! ## Modules
//! - `cpu_exceptions`: Sets up handlers for CPU exceptions (e.g., page fault, double fault).
//! - `hardware_interrupts`: Sets up handlers for hardware IRQs (e.g., timer, keyboard).
//!
//! ## Usage
//! Call `init_idt()` early in kernel initialization to set up the IDT and enable interrupt handling.

#![feature(abi_x86_interrupt)]
#![no_std]

use once_cell::unsync::OnceCell;
use x86_64::structures::idt::InterruptDescriptorTable;

/// CPU exception handler setup (e.g., page fault, double fault).
pub mod cpu_exceptions;
/// Hardware interrupt handler setup (e.g., timer, keyboard).
pub mod hardware_interrupts;

// Static OnceCell for the IDT
static mut IDT: OnceCell<InterruptDescriptorTable> = OnceCell::new();

/// Initialize the Interrupt Descriptor Table (IDT) and load it into the CPU.
///
/// This function sets up all CPU exception and hardware interrupt handlers by
/// initializing the IDT and loading it with the `lidt` instruction. It must be
/// called before enabling interrupts (with `sti`).
///
/// # Safety
/// This function is safe to call once during early kernel initialization, before
/// interrupts are enabled. It uses a static mutable variable, but OnceCell ensures
/// it is only initialized once.
pub fn init_idt() {
    // Safety: single-threaded, called only once before interrupts enabled.
    let idt = unsafe {
        #[allow(static_mut_refs)] // Allowed because OnceCell is used
        IDT.get_or_init(|| {
            let mut idt = InterruptDescriptorTable::new();
            cpu_exceptions::setup_cpu_exceptions(&mut idt);
            hardware_interrupts::setup_hardware_interrupts(&mut idt);
            idt
        })
    };
    idt.load();
}
