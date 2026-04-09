use core::ptr::addr_of;

use x86_64::structures::gdt::SegmentSelector;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

/// IST stack size per CPU (16 KiB).
const IST_STACK_SIZE: usize = 4096 * 4;

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
    static ref BSP_GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());  // 0x08
        let data_selector = gdt.append(Descriptor::kernel_data_segment());  // 0x10
        let tss_selector = gdt.append(Descriptor::tss_segment(&BSP_TSS));   // 0x18 (+ 0x20)
        (gdt, Selectors { code_selector, data_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialize the BSP's GDT, load all segment registers, and load TSS.
/// Called once during boot.
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};

    BSP_GDT.0.load();
    unsafe {
        CS::set_reg(BSP_GDT.1.code_selector);
        DS::set_reg(BSP_GDT.1.data_selector);
        ES::set_reg(BSP_GDT.1.data_selector);
        SS::set_reg(BSP_GDT.1.data_selector);
        load_tss(BSP_GDT.1.tss_selector);
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
        gdt.append(Descriptor::kernel_code_segment());   // 0x08
        gdt.append(Descriptor::kernel_data_segment());    // 0x10
        gdt.append(Descriptor::tss_segment(tss));         // 0x18 (+ 0x20)
        gdt
    }));

    gdt.load();
    unsafe {
        CS::set_reg(SegmentSelector::new(1, x86_64::PrivilegeLevel::Ring0));  // 0x08
        DS::set_reg(SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0));  // 0x10
        ES::set_reg(SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0));
        SS::set_reg(SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0));
        load_tss(SegmentSelector::new(3, x86_64::PrivilegeLevel::Ring0));     // 0x18
    }
}
