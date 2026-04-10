use crate::arch::gdt;
use crate::log;
use crate::log_println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

/// Standard xAPIC base address.
const APIC_BASE: u64 = 0xFEE00000;
/// APIC End-Of-Interrupt register offset.
const APIC_EOI_OFFSET: u64 = 0xB0;

/// APIC error vector (0xFD).
const APIC_ERROR_VECTOR: usize = 0xFD;
/// APIC timer vector (0xFE).
const APIC_TIMER_VECTOR: usize = 0xFE;
/// APIC spurious vector (0xFF).
const APIC_SPURIOUS_VECTOR: usize = 0xFF;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        }
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);

        // LAPIC interrupt handlers — required because the BSP's LAPIC is
        // enabled for IPI delivery. Without these, any LAPIC interrupt
        // hits a not-present IDT entry → #GP → double fault.
        idt[APIC_ERROR_VECTOR as u8].set_handler_fn(apic_error_handler);
        idt[APIC_TIMER_VECTOR as u8].set_handler_fn(apic_timer_handler);
        idt[APIC_SPURIOUS_VECTOR as u8].set_handler_fn(apic_spurious_handler);

        idt
    };
}

pub fn init_exceptions() {
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "IDT: Setting up exception handling");
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    let address = x86_64::registers::control::Cr2::read().expect("Failed to read CR2").as_u64();
    panic!("EXCEPTION: PAGE FAULT: 0x{:x}\n{:#?}\n{:#?}", address, error_code, stack_frame);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: ALIGNMENT CHECK FAULT\n{:#?}\n{:#?}", error_code, stack_frame);
}

/// Send End-Of-Interrupt to the local APIC.
fn apic_eoi() {
    unsafe { core::ptr::write_volatile((APIC_BASE + APIC_EOI_OFFSET) as *mut u32, 0) };
}

extern "x86-interrupt" fn apic_error_handler(_stack_frame: InterruptStackFrame) {
    log_println!(log::SubSystem::X86_64, log::LogLevel::Error, "APIC: error interrupt");
    apic_eoi();
}

extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {
    apic_eoi();
}

/// Spurious interrupts must NOT send EOI (Intel SDM Vol 3, 10.9).
extern "x86-interrupt" fn apic_spurious_handler(_stack_frame: InterruptStackFrame) {
}
