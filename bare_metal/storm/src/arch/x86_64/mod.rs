//! x86_64 architecture support.
//!
//! GDT/TSS, IDT exception handlers, APIC/SMP, SYSCALL/SYSRET,
//! 4-level page tables, PM timer, QEMU integration.

pub mod gdt;
pub mod interrupts;
pub mod apic;
pub mod ap_trampoline;
pub mod syscall;
pub mod page_tables;
pub mod memory_setup;
pub mod timer;
pub mod qemu;

use bootloader_api::info::{MemoryRegions, Optional};

use crate::{log, log_println};

/// Initialize the BSP's CPU state: GDT, TSS, syscall MSRs.
pub fn init_cpu() {
    gdt::init();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "GDT loaded with kernel code/data/user segments and TSS");
    syscall::init();
}

/// Set up exception and interrupt handlers (IDT).
pub fn init_interrupts() {
    interrupts::init_exceptions();
}

/// Set up identity mapping and decouple from bootloader page tables.
/// Returns (physical_memory_offset, kernel_leaf_pages) for later bootloader memory reclaim.
pub fn init_memory(
    physical_memory_offset: u64,
    memory_regions: &MemoryRegions,
    framebuffer_physical: Option<(u64, usize)>,
) -> alloc::vec::Vec<u64> {
    memory_setup::init_identity_mapping(physical_memory_offset, memory_regions);

    crate::physical_memory::init(memory_regions);

    memory_setup::fix_null_guard(physical_memory_offset);
    memory_setup::pre_allocate_kernel_virtual_l3_tables(physical_memory_offset);

    let kernel_leaf_pages = if let Some((physical_address, size)) = framebuffer_physical {
        memory_setup::decouple_from_bootloader(physical_memory_offset, physical_address, size)
    } else {
        memory_setup::decouple_from_bootloader(physical_memory_offset, 0, 0)
    };

    page_tables::save_kernel_cr3();

    kernel_leaf_pages
}

/// Discover and start application processors. Initializes per-CPU
/// syscall state sized for the actual number of CPUs found.
pub fn start_application_processors(rsdp_address: Optional<u64>) {
    let cpu_count = apic::init(rsdp_address);
    syscall::init_per_cpu_state(cpu_count);
    crate::scheduler::idle::init(cpu_count);
}

/// Translate a virtual address to physical using the given page table offset.
/// Used during early boot before identity mapping is active.
pub fn virtual_to_physical(virtual_address: u64, physical_memory_offset: u64) -> Option<u64> {
    page_tables::virtual_to_physical(virtual_address, physical_memory_offset)
}

/// Set the per-CPU kernel stack for syscall entry and TSS RSP0.
pub fn set_thread_kernel_stack(cpu_id: usize, kernel_stack_top: u64) {
    syscall::set_kernel_rsp(cpu_id, kernel_stack_top);
    unsafe { gdt::set_bsp_rsp0(kernel_stack_top); }
}

/// Record which process and thread are running on this CPU.
/// Must be called before entering user mode.
pub fn set_current_context(
    cpu_id: usize,
    process: &crate::process::Process,
    thread: &crate::process::Thread,
) {
    syscall::set_current_context(cpu_id, process, thread);
}

/// Clear the current process/thread on this CPU (on thread exit/yield).
pub fn clear_current_context(cpu_id: usize) {
    syscall::clear_current_context(cpu_id);
}

/// Read the current CPU's hardware ID (LAPIC ID on x86_64).
pub fn cpu_id() -> usize {
    let id_register = unsafe { core::ptr::read_volatile(0xFEE00020u64 as *const u32) };
    (id_register >> 24) as usize
}

/// Low-level context switch. Saves callee-saved registers + RSP to
/// `*old_rsp_ptr`, loads `new_rsp` + registers, returns to new context.
#[unsafe(naked)]
pub extern "C" fn context_switch(_old_rsp_ptr: *mut u64, _new_rsp: u64) {
    core::arch::naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}

/// Bootstrap for new kernel tasks. context_switch `ret`s here.
/// Pops argument and function pointer from the stack, calls the function.
#[unsafe(naked)]
pub extern "C" fn kernel_task_bootstrap() -> ! {
    core::arch::naked_asm!(
        "pop rdi",          // argument (first parameter in System V ABI)
        "pop rax",          // function pointer
        "call rax",         // call the task function (never returns)
        "ud2",              // unreachable
    );
}

/// Switch to a process's address space and jump to user mode.
/// Does not return.
pub fn enter_usermode(entry_point: u64, user_stack_top: u64, page_table_physical: u64) -> ! {
    unsafe {
        // load the process's page tables
        core::arch::asm!("mov cr3, {}", in(reg) page_table_physical, options(nostack));

        // iretq to Ring 3
        let user_cs: u64 = (5 << 3) | 3;  // 0x2B
        let user_ss: u64 = (4 << 3) | 3;  // 0x23
        let user_rflags: u64 = 0x202;      // IF | reserved bit 1
        core::arch::asm!(
            "push {ss}",
            "push {rsp_user}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            ss = in(reg) user_ss,
            rsp_user = in(reg) user_stack_top,
            rflags = in(reg) user_rflags,
            cs = in(reg) user_cs,
            rip = in(reg) entry_point,
            options(noreturn),
        );
    }
}

/// Busy-wait for the given number of milliseconds.
pub fn delay_milliseconds(milliseconds: u64) {
    timer::delay_milliseconds(milliseconds);
}

/// Wait for a keypress or timeout, then exit the emulator.
pub fn wait_or_keypress(seconds: u64) {
    qemu::wait_or_keypress(seconds);
}

/// Exit the emulator with the given code.
pub fn exit_emulator(code: u8) -> ! {
    qemu::exit(code);
}
