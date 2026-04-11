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
mod arch;
mod framebuffer;
mod kernel_memory;
mod log;
mod panic;
mod physical_memory;
mod process;
mod scheduler;
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

    arch::init_cpu();
    arch::init_interrupts();

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
        let physical_address = arch::virtual_to_physical(virtual_address, physical_memory_offset)
            .expect("ramdisk virtual address not mapped");
        log_println!(log::SubSystem::Boot, log::LogLevel::Information,
            "Ramdisk: virtual={:#x}, physical={:#x}, {} bytes", virtual_address, physical_address, length);
        (physical_address, length)
    });

    // framebuffer: walk bootloader page tables to find physical address
    let framebuffer_physical = boot_info.framebuffer.as_ref().map(|framebuffer| {
        let virtual_address = framebuffer.buffer().as_ptr() as u64;
        let size = framebuffer.info().byte_len;
        let physical_address = arch::virtual_to_physical(virtual_address, physical_memory_offset)
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

    // ---- Set up memory: identity mapping, frame allocator, decouple from bootloader ----
    let kernel_leaf_pages = arch::init_memory(physical_memory_offset, &boot_info.memory_regions, framebuffer_physical);

    // sanity check: kernel heap allocator still works after all the page table surgery
    let _heap_test = Box::new(42u64);

    arch::start_application_processors(rsdp_address);

    // ---- Parse ramdisk tar and create user processes ----
    // The ramdisk is a tar archive containing ELF binaries. We iterate
    // entries and create a process for each one. The ramdisk data lives in
    // bootloader-marked physical pages (identity-mapped), so we access it
    // via physical address. create_from_elf copies segment data into each
    // new address space, so the ramdisk can be reclaimed after.
    let mut user_processes = alloc::vec::Vec::new();
    if let Some((physical_address, length)) = ramdisk_physical {
        let ramdisk_data = unsafe {
            core::slice::from_raw_parts(physical_address as *const u8, length)
        };
        match tar_no_std::TarArchiveRef::new(ramdisk_data) {
            Ok(archive) => {
                for entry in archive.entries() {
                    let filename = entry.filename();
                    let name = filename.as_str().unwrap_or("<invalid utf8>");
                    log_println!(log::SubSystem::Boot, log::LogLevel::Information,
                        "Ramdisk: {} ({} bytes)", name, entry.data().len());
                    match process::Process::create_from_elf(entry.data()) {
                        Ok(p) => user_processes.push(p),
                        Err(error) => {
                            log_println!(log::SubSystem::Boot, log::LogLevel::Error,
                                "Failed to load {}: {}", name, error);
                        }
                    }
                }
            }
            Err(error) => {
                log_println!(log::SubSystem::Boot, log::LogLevel::Error,
                    "Failed to parse ramdisk tar: {:?}", error);
            }
        }
    } else {
        log_println!(log::SubSystem::Boot, log::LogLevel::Warning, "No ramdisk present");
    };

    // ---- Reclaim bootloader memory ----
    // Now safe: we're done reading boot_info, ramdisk, and memory_regions.
    // All data we need has been copied to kernel-owned pages.
    physical_memory::reclaim_bootloader(&bootloader_regions[..bootloader_region_count], &kernel_leaf_pages);

    // launch user processes first — they should be scheduled before test threads
    for process in user_processes {
        unsafe {
            PENDING_PROCESSES.lock().push(process);
        }
        scheduler::spawn_kernel(launch_user_process, 0, 0);
    }

    // log sink: drains the log ring buffer to the framebuffer
    scheduler::spawn_kernel(log_sink_task, 0, 0);

    // spawn test kernel threads (including one that never yields — tests preemption)
    for i in 0..3 {
        scheduler::spawn_kernel(test_thread_function, i, 0);
    }
    scheduler::spawn_kernel(spin_forever_function, 99, 0);

    // watchdog: exits QEMU after 5 seconds
    scheduler::spawn_kernel(watchdog_thread, 5, 0);

    log_println!(log::SubSystem::Boot, log::LogLevel::Information,
        "Boot complete — BSP entering scheduler");

    // BSP enters the scheduler as CPU 0
    scheduler::run_on_cpu(0);
}

/// Queue of processes waiting to be launched. Each launcher thread pops one.
static PENDING_PROCESSES: spin::Mutex<alloc::vec::Vec<process::Process>> = spin::Mutex::new(alloc::vec::Vec::new());

/// Kernel thread that takes a pre-loaded Process and jumps to user mode.
fn launch_user_process(_: u64) -> ! {
    let process = PENDING_PROCESSES.lock().pop().expect("no pending process");
    let thread = &process.threads[0];

    log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
        "Jumping to user mode: entry={:#x}, user_stack={:#x}, kernel_stack={:#x}",
        thread.entry_point, thread.user_stack_top, thread.kernel_stack_top);

    // Set up per-CPU state so the syscall handler knows which process/thread
    // is running and can find the kernel stack.
    let cpu_id = arch::cpu_id();
    arch::set_thread_kernel_stack(cpu_id, thread.kernel_stack_top);
    arch::set_current_context(cpu_id, &process, thread);

    // switch to the process's address space and jump to user mode
    arch::enter_usermode(
        thread.entry_point,
        thread.user_stack_top,
        process.address_space.l4_physical_address(),
    );
}

/// Test thread function — logs a message, yields, repeats.
fn test_thread_function(thread_number: u64) -> ! {
    loop {
        log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
            "Thread {} running", thread_number);
        // busy-wait a bit to simulate work (PM timer based)
        arch::delay_milliseconds(500);
        scheduler::yield_current();
    }
}

/// Test thread that never yields — only runs if preemption works.
/// If it monopolizes a CPU, other threads on that CPU won't get scheduled.
fn spin_forever_function(thread_number: u64) -> ! {
    log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
        "Spin thread {} started (never yields)", thread_number);
    loop {
        core::hint::spin_loop();
    }
}

/// Log sink task — drains the log queue and writes to the framebuffer.
fn log_sink_task(_: u64) -> ! {
    log::log_sink_loop();
}

/// Watchdog thread — checks time in a yield loop so it doesn't starve
/// other tasks on single-CPU systems.
fn watchdog_thread(seconds: u64) -> ! {
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    let timeout_ticks = seconds * 1_000_000_000; // ~1 GHz TSC assumed (QEMU default)
    loop {
        let elapsed = unsafe { core::arch::x86_64::_rdtsc() } - start;
        if elapsed >= timeout_ticks {
            arch::exit_emulator(0);
        }
        scheduler::yield_current();
    }
}
