use core::ptr::addr_of;

use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel::Ring0;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

// GDT layout (same for BSP and all APs):
//   0x00  Null
//   0x08  Kernel code  (SYSCALL CS)
//   0x10  Kernel data  (SYSCALL SS = 0x08+8)
//   0x18  User data    (placeholder for 32-bit SYSRET CS — unused)
//   0x20  User data    (SYSRET SS = 0x18+8, RPL=3)
//   0x28  User code 64 (SYSRET CS = 0x18+16, RPL=3)
//   0x30  TSS low
//   0x38  TSS high
//
// STAR MSR: kernel_base=0x08, user_base=0x18
pub const STAR_KERNEL_BASE: u16 = 0x08;
pub const STAR_USER_BASE: u16 = 0x18;

/// IST stack size per CPU (16 KiB).
const IST_STACK_SIZE: usize = 4096 * 4;

/// Build a GDT with the standard segment layout and the given TSS.
/// Returns (gdt, tss_selector).
fn build_gdt(gdt: &mut GlobalDescriptorTable, tss: &'static TaskStateSegment) -> SegmentSelector {
    gdt.append(Descriptor::kernel_code_segment());  // 0x08
    gdt.append(Descriptor::kernel_data_segment());   // 0x10
    gdt.append(Descriptor::user_data_segment());     // 0x18 (32-bit sysret CS slot — not used)
    gdt.append(Descriptor::user_data_segment());     // 0x20 (sysret SS)
    gdt.append(Descriptor::user_code_segment());     // 0x28 (sysret CS, 64-bit)
    gdt.append(Descriptor::tss_segment(tss))         // 0x30 (+ 0x38)
}

/// Create a new TSS with freshly allocated IST stacks.
/// Each CPU must have its own TSS (the TSS descriptor has a "busy" bit
/// that prevents sharing).
fn create_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();

    // allocate IST stacks from kernel virtual memory
    let double_fault_stack = crate::virtual_memory::allocate_contiguous_pages(IST_STACK_SIZE / 4096)
        .expect("Failed to allocate double-fault IST stack");
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
        VirtAddr::new(double_fault_stack as u64 + IST_STACK_SIZE as u64);

    let page_fault_stack = crate::virtual_memory::allocate_contiguous_pages(IST_STACK_SIZE / 4096)
        .expect("Failed to allocate page-fault IST stack");
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] =
        VirtAddr::new(page_fault_stack as u64 + IST_STACK_SIZE as u64);

    tss
}

// BSP's GDT and TSS — initialized once at boot via lazy_static
lazy_static! {
    static ref BSP_TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // BSP uses static stacks (available before virtual_memory is ready)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 4;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE as u64
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 4;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

lazy_static! {
    static ref BSP_GDT: (GlobalDescriptorTable, SegmentSelector) = {
        let mut gdt = GlobalDescriptorTable::new();
        let tss_selector = build_gdt(&mut gdt, &BSP_TSS);
        (gdt, tss_selector)
    };
}

/// Set TSS.privilege_stack_table[0] (RSP0) for the BSP.
/// The CPU loads this into RSP when transitioning from Ring 3 to Ring 0
/// via an interrupt or exception. Must be called before entering user mode.
///
/// Safety: mutates the BSP TSS through a shared reference. Only call from
/// the BSP before entering user mode.
pub unsafe fn set_bsp_rsp0(rsp0: u64) {
    let tss_ptr = &*BSP_TSS as *const TaskStateSegment as *mut TaskStateSegment;
    (*tss_ptr).privilege_stack_table[0] = VirtAddr::new(rsp0);
}

/// Initialize the BSP's GDT, load all segment registers, and load TSS.
/// Called once during boot.
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};

    BSP_GDT.0.load();
    unsafe {
        CS::set_reg(SegmentSelector::new(1, Ring0));   // 0x08
        DS::set_reg(SegmentSelector::new(2, Ring0));   // 0x10
        ES::set_reg(SegmentSelector::new(2, Ring0));
        SS::set_reg(SegmentSelector::new(2, Ring0));
        load_tss(BSP_GDT.1);                           // 0x30
    }
}

/// Initialize a per-AP GDT with its own TSS and load it.
/// Called from ap_entry on each AP after the trampoline has loaded
/// the BSP's GDT (for initial segment setup) and the kernel IDT.
pub fn init_ap() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};

    // Each AP needs its own GDT+TSS because the TSS descriptor's "busy"
    // bit prevents sharing. We leak the allocation — it lives forever.
    let tss = alloc::boxed::Box::leak(alloc::boxed::Box::new(create_tss()));
    let gdt = alloc::boxed::Box::leak(alloc::boxed::Box::new({
        let mut gdt = GlobalDescriptorTable::new();
        let _tss_selector = build_gdt(&mut gdt, tss);
        gdt
    }));

    gdt.load();
    unsafe {
        CS::set_reg(SegmentSelector::new(1, Ring0));   // 0x08
        DS::set_reg(SegmentSelector::new(2, Ring0));   // 0x10
        ES::set_reg(SegmentSelector::new(2, Ring0));
        SS::set_reg(SegmentSelector::new(2, Ring0));
        load_tss(SegmentSelector::new(6, Ring0));      // 0x30
    }
}
