use polished_interrupts::init_idt;
use polished_serial_logging::info;

pub fn init_interrupts() {
    info("Loading IDT...");
    init_idt();
    info("IDT loaded");
}
