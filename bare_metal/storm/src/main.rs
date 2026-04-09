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
mod scheduler;
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

    // ---- Snapshot everything from boot_info BEFORE modifying page tables ----
    // The bootloader's page tables contain mappings for boot_info, ramdisk,
    // and framebuffer at virtual addresses we won't preserve. Extract all
    // physical addresses now using the bootloader's page tables, then use
    // identity-mapped access after we install our own page tables.
    let rsdp_address = boot_info.rsdp_addr;
    log_println!(log::SubSystem::Boot, log::LogLevel::Debug, "RSDP address: {:?}", rsdp_address);

    // ramdisk: walk bootloader page tables to find physical address
    let ramdisk_physical = boot_info.ramdisk_addr.into_option().map(|virtual_address| {
        let length = boot_info.ramdisk_len as usize;
        let physical_address = page_tables::virtual_to_physical(virtual_address, physical_memory_offset)
            .expect("ramdisk virtual address not mapped");
        log_println!(log::SubSystem::Boot, log::LogLevel::Information,
            "Ramdisk: virtual={:#x}, physical={:#x}, {} bytes", virtual_address, physical_address, length);
        (physical_address, length)
    });

    // framebuffer: walk bootloader page tables to find physical address
    let framebuffer_physical = boot_info.framebuffer.as_ref().map(|framebuffer| {
        let virtual_address = framebuffer.buffer().as_ptr() as u64;
        let size = framebuffer.info().byte_len;
        let physical_address = page_tables::virtual_to_physical(virtual_address, physical_memory_offset)
            .expect("framebuffer virtual address not mapped");
        log_println!(log::SubSystem::Boot, log::LogLevel::Debug,
            "Framebuffer: virtual={:#x}, physical={:#x}, size={} KiB", virtual_address, physical_address, size / 1024);
        (physical_address, size)
    });

    // snapshot bootloader region ranges (memory_regions itself lives in bootloader memory)
    let mut bootloader_regions = [(0u64, 0u64); 8];
    let mut bootloader_region_count = 0;
    for region in boot_info.memory_regions.iter() {
        if region.kind == bootloader_api::info::MemoryRegionKind::Bootloader && bootloader_region_count < bootloader_regions.len() {
            bootloader_regions[bootloader_region_count] = (region.start, region.end);
            bootloader_region_count += 1;
        }
    }

    // ---- Now safe to modify page tables ----
    memory_setup::init_identity_mapping(physical_memory_offset, &boot_info.memory_regions);

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
    // NOTE: boot_info DATA pages are still accessible — only page table pages
    // are moved. We can keep reading boot_info until we reclaim.
    let kernel_leaf_pages = if let Some((physical_address, size)) = framebuffer_physical {
        memory_setup::decouple_from_bootloader(physical_memory_offset, physical_address, size)
    } else {
        memory_setup::decouple_from_bootloader(physical_memory_offset, 0, 0)
    };

    // sanity check: kernel heap allocator still works after all the page table surgery
    let _heap_test = Box::new(42u64);

    apic::init(rsdp_address);

    // initialize syscall MSRs on BSP
    syscall::init();

    // ---- Load ramdisk ELF and create user process ----
    // The ramdisk data lives in bootloader-marked physical pages that are
    // still identity-mapped after decouple. We access it via the physical
    // address (snapshotted above). create_from_elf copies all segment data
    // into the new address space, so the ramdisk memory can be reclaimed
    // immediately after.
    let user_process = if let Some((physical_address, length)) = ramdisk_physical {
        let elf_data = unsafe {
            core::slice::from_raw_parts(physical_address as *const u8, length)
        };
        match process::Process::create_from_elf(elf_data) {
            Ok(p) => Some(p),
            Err(error) => {
                log_println!(log::SubSystem::Boot, log::LogLevel::Error,
                    "Failed to load ramdisk ELF: {}", error);
                None
            }
        }
    } else {
        log_println!(log::SubSystem::Boot, log::LogLevel::Warning, "No ramdisk present");
        None
    };

    // ---- Reclaim bootloader memory ----
    // Now safe: we're done reading boot_info, ramdisk, and memory_regions.
    // All data we need has been copied to kernel-owned pages.
    physical_memory::reclaim_bootloader(&bootloader_regions[..bootloader_region_count], &kernel_leaf_pages);

    // spawn test kernel threads
    for i in 0..4 {
        scheduler::spawn(test_thread_function, i);
    }

    // watchdog: exits QEMU after 10 seconds (or keypress)
    scheduler::spawn(watchdog_thread, 10);

    // launch the user process
    if let Some(process) = user_process {
        // Store process in a static so the launcher thread can take it.
        unsafe { PENDING_PROCESS = Some(process) };
        scheduler::spawn(launch_user_process, 0);
    }

    log_println!(log::SubSystem::Boot, log::LogLevel::Information,
        "Boot complete — BSP entering scheduler");

    // BSP enters the scheduler as CPU 0
    scheduler::run_on_cpu(0);
}

/// Process created in kernel_main, consumed by launch_user_process thread.
static mut PENDING_PROCESS: Option<process::Process> = None;

/// Kernel thread that takes a pre-loaded Process and jumps to user mode.
fn launch_user_process(_: u64) -> ! {
    let process = unsafe {
        PENDING_PROCESS.take().expect("no pending process")
    };

    log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
        "Jumping to user mode: entry={:#x}, stack={:#x}", process.entry_point, process.user_stack_top);

    // allocate a kernel stack for this process (used on syscall re-entry
    // and by the CPU for ring 3→0 transitions via TSS.RSP0)
    let kernel_stack = virtual_memory::allocate_contiguous_pages(16)
        .expect("Failed to allocate process kernel stack");
    let kernel_stack_top = kernel_stack as u64 + (16 * 0x1000);
    syscall::set_kernel_rsp(kernel_stack_top);

    // set TSS.RSP0 so the CPU switches to this kernel stack when handling
    // interrupts/exceptions from Ring 3
    unsafe { gdt::set_bsp_rsp0(kernel_stack_top); }

    // switch to the process's address space
    let cr3_value = process.address_space.l4_physical_address();
    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "Switching CR3 to {:#x}", cr3_value);
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) cr3_value, options(nostack));
    }

    // verify kernel still works after CR3 switch
    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "CR3 switched, kernel still running");

    // verify user entry point is readable from kernel
    let entry_byte = unsafe { core::ptr::read_volatile(process.entry_point as *const u8) };
    log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
        "User entry byte: {:#x}", entry_byte);

    // jump to user mode via IRETQ
    // Build an interrupt return frame on the kernel stack and iretq to Ring 3.
    // Stack frame (pushed in reverse): SS, RSP, RFLAGS, CS, RIP
    let user_cs: u64 = (5 << 3) | 3;  // GDT index 5, RPL=3 = 0x2B
    let user_ss: u64 = (4 << 3) | 3;  // GDT index 4, RPL=3 = 0x23
    let user_rflags: u64 = 0x202;      // IF | reserved bit 1
    unsafe {
        core::arch::asm!(
            "push {ss}",
            "push {rsp_user}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            ss = in(reg) user_ss,
            rsp_user = in(reg) process.user_stack_top,
            rflags = in(reg) user_rflags,
            cs = in(reg) user_cs,
            rip = in(reg) process.entry_point,
            options(noreturn),
        );
    }
}

/// Test thread function — logs a message, yields, repeats.
fn test_thread_function(thread_number: u64) -> ! {
    loop {
        log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
            "Thread {} running", thread_number);
        // busy-wait a bit to simulate work (PM timer based)
        timer::delay_milliseconds(500);
        scheduler::yield_thread();
    }
}

/// Watchdog thread — waits for the given number of seconds (or keypress), then exits QEMU.
fn watchdog_thread(seconds: u64) -> ! {
    qemu::wait_or_keypress(seconds);
    qemu::exit(0);
}
