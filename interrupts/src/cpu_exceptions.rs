use core::arch::asm;

use serial_logging::kprint;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

pub fn setup_cpu_exceptions(idt: &mut InterruptDescriptorTable) {
    // Set IST index for double fault (IST1)
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(1);
        // Set IST index for NMI (IST2)
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt_handler)
            .set_stack_index(2);
    }
    // Other exceptions can be set similarly if needed
    idt.divide_error.set_handler_fn(divide_by_zero_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.bound_range_exceeded
        .set_handler_fn(bound_range_exceeded_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available
        .set_handler_fn(device_not_available_handler);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.x87_floating_point
        .set_handler_fn(x87_floating_point_handler);
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    idt.machine_check.set_handler_fn(machine_check_handler);
    idt.simd_floating_point
        .set_handler_fn(simd_floating_point_handler);
    idt.virtualization
        .set_handler_fn(virtualization_exception_handler);
}

pub extern "x86-interrupt" fn divide_by_zero_handler(_stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] EXCEPTION: DIVIDE BY ZERO\r\n");
    kprint!(
        "[SUGGESTION] Possible cause: Division by zero. Solution: Check divisor before division.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprint!("[ERROR] General Protection Fault: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:#x}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Invalid memory access or segment. Solution: Check segment selectors and memory accesses.\r\n"
    );
    panic!("General Protection Fault: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    kprint!(
        "[SUGGESTION] Possible cause: Exception during exception handling. Solution: Check stack overflows and handler correctness.\r\n"
    );
    panic!("Double Fault: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    kprint!("[DEBUG] Debug Exception: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Debug exception (breakpoint, single-step). Solution: Check debug registers and breakpoints.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    kprint!("[NMI] Non-Maskable Interrupt: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Hardware failure or NMI source. Solution: Check hardware and NMI sources.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprint!("[DEBUG] Breakpoint Exception: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Breakpoint instruction (int3). Solution: Check for intentional breakpoints.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] Stack Overflow: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: INTO instruction overflow. Solution: Check arithmetic operations for overflow.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] Bound Range Exceeded: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: BOUND instruction out of range. Solution: Check array bounds.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] Invalid Opcode: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Invalid or undefined instruction. Solution: Check for unsupported CPU instructions.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] Device Not Available: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: FPU or device not available. Solution: Check FPU usage and TS flag.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprint!("[ERROR] Invalid TSS: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:#x}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Invalid Task State Segment. Solution: Check TSS setup and task switching.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprint!("[ERROR] Segment Not Present: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:#x}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Segment not present in memory. Solution: Check segment descriptors.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprint!("[ERROR] Stack Segment Fault: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:#x}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Stack segment error. Solution: Check stack pointers and segment limits.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

use x86_64::structures::idt::PageFaultErrorCode;

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    kprint!("[ERROR] Page Fault: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:?}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Invalid memory access. Solution: Check page tables and memory accesses.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprint!(
        "[ERROR] x87 Floating Point Exception: {:#?}\r\n",
        stack_frame
    );
    kprint!(
        "[SUGGESTION] Possible cause: x87 FPU error. Solution: Check floating point operations.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprint!("[ERROR] Alignment Check Exception: {:#?}\r\n", stack_frame);
    kprint!("[ERROR] Error Code: {:#x}\r\n", error_code);
    kprint!(
        "[SUGGESTION] Possible cause: Unaligned memory access. Solution: Check data alignment.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    kprint!("[ERROR] Machine Check Exception: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Hardware error. Solution: Check hardware status and logs.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprint!(
        "[ERROR] SIMD Floating Point Exception: {:#?}\r\n",
        stack_frame
    );
    kprint!("[SUGGESTION] Possible cause: SIMD FPU error. Solution: Check SIMD operations.\r\n");
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

pub extern "x86-interrupt" fn virtualization_exception_handler(stack_frame: InterruptStackFrame) {
    kprint!("[ERROR] Virtualization Exception: {:#?}\r\n", stack_frame);
    kprint!(
        "[SUGGESTION] Possible cause: Virtualization instruction error. Solution: Check virtualization support and usage.\r\n"
    );
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}
