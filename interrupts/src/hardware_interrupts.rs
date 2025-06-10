use serial_logging::kprint;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[INFO] INT 0x20: Timer interrupt\r\n");
    // TODO: Send EOI to the PIC if using legacy PIC
}
