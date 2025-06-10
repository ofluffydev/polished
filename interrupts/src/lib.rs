#![feature(abi_x86_interrupt)]
#![no_std]
use once_cell::unsync::OnceCell;
use x86_64::structures::idt::InterruptDescriptorTable;

pub mod cpu_exceptions;
pub mod hardware_interrupts;

// Static OnceCell for the IDT
static mut IDT: OnceCell<InterruptDescriptorTable> = OnceCell::new();

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
