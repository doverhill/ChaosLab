use crate::gdt;
use crate::serial_println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler).set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        }
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt
    };
}

pub fn init_exceptions() {
    serial_println!("Setting up exception handling");
    IDT.load();
    // x86_64::instructions::interrupts::disable();
    // x86_64::instructions::interrupts::int3(); // new
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    let address = x86_64::registers::control::Cr2::read().as_u64();
    serial_println!("pf");
    panic!("EXCEPTION: PAGE FAULT: 0x{:x}\n{:#?}\n{:#?}", address, error_code, stack_frame);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: ALIGNMENT CHECK FAULT\n{:#?}\n{:#?}", error_code, stack_frame);
}
