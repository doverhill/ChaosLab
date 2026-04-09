// use virtual address space:
// bottom half is physical memory identity mapped so that virtual addresses with top bit = 0 are physical
// upper half is virtual per process address space

// 1. create a kernel address space that only contains the bottom half identity mapped. this address space is used by all kernel threads. these page tables will also be reused by all process address spaces (these page tables)
// 2. set up irq and exception handlers
// 3. set up syscalls
// 3. start all processors
// each processor has its own process list
// 4. parse ramdisk and start a process for each provided elf image

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod address_space;
mod ap_trampoline;
mod apic;
mod framebuffer;
mod gdt;
mod interrupts;
mod kernel_memory;
mod log;
mod memory_setup;
mod page_tables;
mod panic;
mod physical_memory;
mod process;
mod qemu;
mod syscall;
mod timer;
mod virtual_memory;

use alloc::boxed::Box;
#[allow(deprecated)]
use bootloader_api::config::FrameBuffer;
use bootloader_api::{config::Mapping, config::Mappings, entry_point, BootloaderConfig};

pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;
pub const GB: usize = 1024 * 1024 * 1024;

#[allow(deprecated)]
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings = Mappings::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 128 * 1024;
    let mut framebuffer = FrameBuffer::new_default();
    framebuffer.minimum_framebuffer_width = Some(800);
    framebuffer.minimum_framebuffer_height = Some(600);
    config.frame_buffer = framebuffer;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    log::init_boot_tsc();
    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Starting RustStorm kernel");

    // extract framebuffer for screen output
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        log_println!(
            log::SubSystem::Boot,
            log::LogLevel::Information,
            "Framebuffer: {}x{} {:?} {}bpp (stride {})",
            info.width,
            info.height,
            info.pixel_format,
            info.bytes_per_pixel * 8,
            info.stride
        );

        // obtain a 'static buffer reference independent of boot_info borrow
        let buffer_pointer = framebuffer.buffer_mut().as_mut_ptr();
        let len = info.byte_len;
        let buffer = unsafe { core::slice::from_raw_parts_mut(buffer_pointer, len) };

        let writer = framebuffer::FramebufferWriter::new(buffer, info);
        log_println!(
            log::SubSystem::Boot,
            log::LogLevel::Information,
            "Screen console: {}x{} chars",
            writer.cols(),
            writer.rows()
        );
        log::init_framebuffer(writer);
        log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Framebuffer logger active");
    }

    gdt::init();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "GDT loaded with kernel code segment and TSS");

    interrupts::init_exceptions();

    let physical_memory_offset = boot_info.physical_memory_offset.into_option().expect("bootloader did not provide physical memory offset");
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "Physical memory offset: {:#x} (L4[{}])", physical_memory_offset, physical_memory_offset / (512 * crate::GB as u64));
    memory_setup::init_identity_mapping(physical_memory_offset, &boot_info.memory_regions);

    // Save everything we need from boot_info BEFORE physical_memory::init.
    // boot_info and memory_regions live in Bootloader-marked pages which
    // will be reclaimed after decoupling.
    let rsdp_address = boot_info.rsdp_addr;
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "RSDP address: {:?}", rsdp_address);

    // Snapshot framebuffer physical address (needed for identity mapping).
    // Walk the page tables to find the actual physical address backing the
    // framebuffer virtual pointer (the bootloader may use a separate mapping).
    let framebuffer_physical = boot_info.framebuffer.as_ref().map(|framebuffer| {
        let virtual_address = framebuffer.buffer().as_ptr() as u64;
        let size = framebuffer.info().byte_len;
        let physical_address = page_tables::virtual_to_physical(virtual_address, physical_memory_offset)
            .expect("framebuffer virtual address not mapped");
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "Framebuffer: virtual={:#x}, physical={:#x}, size={} KiB", virtual_address, physical_address, size / 1024);
        (physical_address, size)
    });

    // snapshot bootloader region ranges to the stack (memory_regions itself is in bootloader memory)
    let mut bootloader_regions = [(0u64, 0u64); 8];
    let mut bootloader_region_count = 0;
    for region in boot_info.memory_regions.iter() {
        if region.kind == bootloader_api::info::MemoryRegionKind::Bootloader && bootloader_region_count < bootloader_regions.len() {
            bootloader_regions[bootloader_region_count] = (region.start, region.end);
            bootloader_region_count += 1;
        }
    }

    // initialize frame allocator with Usable memory only
    physical_memory::init(&boot_info.memory_regions);

    // Fix null guard FIRST — the heap allocator uses virtual_memory which
    // walks page tables via identity mapping. The L4 table at 0x101000 must
    // be accessible before any heap allocation can occur.
    memory_setup::fix_null_guard(physical_memory_offset);

    // Pre-allocate L3 tables for kernel virtual memory (L4[128..256]) so that
    // all future address spaces can share the kernel half by copying L4 entries.
    memory_setup::pre_allocate_kernel_virtual_l3_tables(physical_memory_offset);

    // ---- Decouple from bootloader page tables ----
    // After this, no page table pages are in bootloader memory.
    // The framebuffer gets identity-mapped and its pointer updated.
    let kernel_leaf_pages = if let Some((physical_address, size)) = framebuffer_physical {
        memory_setup::decouple_from_bootloader(physical_memory_offset, physical_address, size)
    } else {
        memory_setup::decouple_from_bootloader(physical_memory_offset, 0, 0)
    };

    // ---- Reclaim bootloader memory ----
    physical_memory::reclaim_bootloader(&bootloader_regions[..bootloader_region_count], &kernel_leaf_pages);

    // sanity check: kernel heap allocator still works after all the page table surgery
    let _heap_test = Box::new(42u64);

    // Dump BSP register state so we can replicate on APs
    unsafe {
        let cs: u16; let ss: u16; let ds: u16; let es: u16; let fs: u16; let gs: u16;
        core::arch::asm!("mov {:x}, cs", out(reg) cs, options(nomem, nostack));
        core::arch::asm!("mov {:x}, ss", out(reg) ss, options(nomem, nostack));
        core::arch::asm!("mov {:x}, ds", out(reg) ds, options(nomem, nostack));
        core::arch::asm!("mov {:x}, es", out(reg) es, options(nomem, nostack));
        core::arch::asm!("mov {:x}, fs", out(reg) fs, options(nomem, nostack));
        core::arch::asm!("mov {:x}, gs", out(reg) gs, options(nomem, nostack));
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "BSP segments: CS={:#x} SS={:#x} DS={:#x} ES={:#x} FS={:#x} GS={:#x}",
            cs, ss, ds, es, fs, gs);

        // dump GDT and IDT base/limit
        let mut gdt_buf = [0u8; 10];
        let mut idt_buf = [0u8; 10];
        core::arch::asm!("sgdt [{}]", in(reg) gdt_buf.as_mut_ptr(), options(nostack));
        core::arch::asm!("sidt [{}]", in(reg) idt_buf.as_mut_ptr(), options(nostack));
        let gdt_limit = u16::from_le_bytes([gdt_buf[0], gdt_buf[1]]);
        let gdt_base = u64::from_le_bytes([gdt_buf[2], gdt_buf[3], gdt_buf[4], gdt_buf[5], gdt_buf[6], gdt_buf[7], gdt_buf[8], gdt_buf[9]]);
        let idt_limit = u16::from_le_bytes([idt_buf[0], idt_buf[1]]);
        let idt_base = u64::from_le_bytes([idt_buf[2], idt_buf[3], idt_buf[4], idt_buf[5], idt_buf[6], idt_buf[7], idt_buf[8], idt_buf[9]]);
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "BSP GDT: base={:#x} limit={:#x}", gdt_base, gdt_limit);
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "BSP IDT: base={:#x} limit={:#x}", idt_base, idt_limit);
    }

    apic::init(rsdp_address);

    // test: create a process with its own address space and allocate user pages
    let mut test_process = process::Process::create();
    if let Some(user_pages) = test_process.address_space.allocate_user_pages(4) {
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "Process {} allocated 4 user pages at {:#x}", test_process.process_id, user_pages);
    }
    test_process.create_thread();
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
        "Process {} has {} threads", test_process.process_id, test_process.threads.len());

    log_println!(log::SubSystem::Boot, log::LogLevel::Information, "Boot complete — press any key or wait 20s");
    qemu::wait_or_keypress(20);
    qemu::exit(0);
}
