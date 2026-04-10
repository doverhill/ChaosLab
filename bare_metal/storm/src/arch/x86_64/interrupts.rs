use crate::arch::gdt;
use crate::{arch, log, log_println, scheduler};
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
/// Scheduler IPI vector — used to wake idle CPUs from HLT.
const SCHEDULER_IPI_VECTOR: usize = 0xFC;

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

        // LAPIC interrupt handlers
        idt[APIC_ERROR_VECTOR as u8].set_handler_fn(apic_error_handler);
        // Timer handler uses a naked wrapper that saves ALL GPRs for reliable
        // context switching. We set the raw handler address in the IDT entry.
        unsafe {
            idt[APIC_TIMER_VECTOR as u8].set_handler_addr(
                x86_64::VirtAddr::new(apic_timer_handler_naked as *const () as u64),
            );
        }
        idt[APIC_SPURIOUS_VECTOR as u8].set_handler_fn(apic_spurious_handler);
        idt[SCHEDULER_IPI_VECTOR as u8].set_handler_fn(scheduler_ipi_handler);

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

/// Naked APIC timer handler wrapper. Saves ALL GPRs explicitly (the
/// x86-interrupt ABI doesn't guarantee this when we call context_switch
/// from inside the handler). The CPU already pushed the interrupt frame
/// (SS, RSP, RFLAGS, CS, RIP) before we get here.
///
/// Stack after our pushes:
///   [interrupt frame]  ← pushed by CPU
///   [rax..r15]         ← pushed by us (15 GPRs)
///   [context_switch callee-saved + return addr]  ← pushed by context_switch if preemption happens
///
/// On resume after preemption: context_switch ret's here, we pop all
/// GPRs, iretq resumes the interrupted code.
#[unsafe(naked)]
extern "C" fn apic_timer_handler_naked() {
    core::arch::naked_asm!(
        // Save all general-purpose registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call the Rust handler. It may context_switch away (preemption)
        // and return much later when this task is dispatched again.
        "call {handler}",

        // Restore all GPRs (either immediately or after being re-dispatched)
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "iretq",

        handler = sym apic_timer_handler_rust,
    );
}

/// Rust portion of the timer handler. Called from the naked wrapper
/// with all GPRs saved on the stack.
extern "C" fn apic_timer_handler_rust() {
    apic_eoi();

    let cpu_id = arch::cpu_id();

    // Is a task running on this CPU?
    let task_id = match scheduler::idle::get_current_task_id(cpu_id) {
        Some(id) => id,
        None => return,
    };

    // Are there other tasks waiting to run?
    let should_preempt = {
        let state = scheduler::SCHEDULER.lock();
        !state.run_queue.is_empty()
    };

    if !should_preempt {
        return;
    }

    // Preempt: context_switch to the idle loop. The naked wrapper already
    // saved all GPRs. context_switch adds its own callee-saved frame on top.
    // When this task is later dispatched, context_switch ret's back here,
    // we return to the naked wrapper which pops all GPRs and does iretq.
    let idle_rsp = unsafe { *(scheduler::idle::get_idle_rsp(cpu_id) as *const u64) };
    let task_rsp_ptr = {
        let mut state = scheduler::SCHEDULER.lock();
        match state.get_mut(task_id) {
            Some(task) => &mut task.saved_rsp as *mut u64,
            None => return,
        }
    };

    unsafe {
        arch::context_switch(task_rsp_ptr, idle_rsp);
    }
    // Resumed here when dispatched again — return to naked wrapper
}

/// Spurious interrupts must NOT send EOI (Intel SDM Vol 3, 10.9).
extern "x86-interrupt" fn apic_spurious_handler(_stack_frame: InterruptStackFrame) {
}

/// Scheduler IPI handler — wakes a CPU from HLT. Just EOI, the idle loop
/// handles the rest.
extern "x86-interrupt" fn scheduler_ipi_handler(_stack_frame: InterruptStackFrame) {
    apic_eoi();
}
