# Polished Interrupts TODO List

This document tracks the status and planned features for interrupt handling in Polished OS. For project context, see the [main README](../README.md).

______________________________________________________________________

## 1. CPU Exceptions (Faults and Traps)

- [x] Set up handler: Divide-by-zero (#DE)
- [x] Set up handler: Debug (#DB)
- [x] Set up handler: Non-maskable interrupt (NMI)
- [x] Set up handler: Breakpoint (#BP)
- [x] Set up handler: Overflow
- [x] Set up handler: Bound range exceeded
- [x] Set up handler: Invalid opcode
- [x] Set up handler: Device not available
- [x] Set up handler: Double fault (with dedicated IST stack)
- [x] Set up handler: Invalid TSS
- [x] Set up handler: Segment not present
- [x] Set up handler: Stack-segment fault
- [x] Set up handler: General protection fault (#GP)
- [x] Set up handler: Page fault (#PF)
- [x] Set up handler: x87 Floating-point error
- [x] Set up handler: Alignment check
- [x] Set up handler: Machine check
- [x] Set up handler: SIMD floating-point exception
- [x] Set up handler: Virtualization exception

## 2. Hardware Interrupts (IRQs)

- [x] Set up handler: Timer (PIT/APIC)
- [x] Set up handler: Keyboard
- [x] Set up handler: Mouse
- [x] Set up handler: Disk controllers (SATA, NVMe)
- [x] Set up handler: Network cards
- [x] Set up handler: USB controllers
- [x] Set up handler: Other hardware devices as present

## 3. Software Interrupts (Syscalls)

- [ ] Set up syscall interrupt vector (e.g., int 0x80) if needed
- [ ] Set up syscall entry point for `syscall` instruction

Will use the modern syscall mechanism instead of legacy interrupts.

## 4. Stack Management

- [ ] Configure Interrupt Stack Table (IST) for double fault
- [ ] Configure IST for NMI
- [ ] Configure IST for other critical exceptions as needed

## 5. Interrupt Controller Initialization

- [ ] Initialize and configure APIC or legacy PIC
- [ ] Set up IRQ vector remapping
- [ ] Mask/unmask interrupts as needed
- [ ] Implement End-of-Interrupt (EOI) signaling in handlers

______________________________________________________________________

This list is updated as features are implemented. Feedback and contributions are welcome!
