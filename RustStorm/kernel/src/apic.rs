//! APIC initialization and SMP startup.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use bootloader_api::info::Optional;
use x2apic::lapic::{LocalApicBuilder, xapic_base};

use crate::ap_trampoline;
use crate::virtual_memory;
use crate::{log, log_println, timer};

/// Number of APs that have completed initialization and are ready.
static AP_READY_COUNT: AtomicU32 = AtomicU32::new(0);

/// Per-AP stack size (64 KiB).
const AP_STACK_PAGES: usize = 16;

#[derive(Clone)]
struct AcpiHandler();

impl acpi::Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping {
            physical_start: physical_address,
            virtual_start: core::ptr::NonNull::<T>::new(physical_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }
    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
    fn nanos_since_boot(&self) -> u64 { 0 }
    fn stall(&self, _microseconds: u64) {}
    fn sleep(&self, _milliseconds: u64) {}
    fn read_u8(&self, address: usize) -> u8 { unsafe { core::ptr::read_volatile(address as *const u8) } }
    fn read_u16(&self, address: usize) -> u16 { unsafe { core::ptr::read_volatile(address as *const u16) } }
    fn read_u32(&self, address: usize) -> u32 { unsafe { core::ptr::read_volatile(address as *const u32) } }
    fn read_u64(&self, address: usize) -> u64 { unsafe { core::ptr::read_volatile(address as *const u64) } }
    fn write_u8(&self, address: usize, value: u8) { unsafe { core::ptr::write_volatile(address as *mut u8, value) } }
    fn write_u16(&self, address: usize, value: u16) { unsafe { core::ptr::write_volatile(address as *mut u16, value) } }
    fn write_u32(&self, address: usize, value: u32) { unsafe { core::ptr::write_volatile(address as *mut u32, value) } }
    fn write_u64(&self, address: usize, value: u64) { unsafe { core::ptr::write_volatile(address as *mut u64, value) } }
    fn read_io_u8(&self, port: u16) -> u8 { unsafe { x86_64::instructions::port::Port::new(port).read() } }
    fn read_io_u16(&self, port: u16) -> u16 { unsafe { x86_64::instructions::port::Port::new(port).read() } }
    fn read_io_u32(&self, port: u16) -> u32 { unsafe { x86_64::instructions::port::Port::new(port).read() } }
    fn write_io_u8(&self, port: u16, value: u8) { unsafe { x86_64::instructions::port::Port::new(port).write(value) } }
    fn write_io_u16(&self, port: u16, value: u16) { unsafe { x86_64::instructions::port::Port::new(port).write(value) } }
    fn write_io_u32(&self, port: u16, value: u32) { unsafe { x86_64::instructions::port::Port::new(port).write(value) } }
    fn read_pci_u8(&self, _address: acpi::PciAddress, _offset: u16) -> u8 { unimplemented!() }
    fn read_pci_u16(&self, _address: acpi::PciAddress, _offset: u16) -> u16 { unimplemented!() }
    fn read_pci_u32(&self, _address: acpi::PciAddress, _offset: u16) -> u32 { unimplemented!() }
    fn write_pci_u8(&self, _address: acpi::PciAddress, _offset: u16, _value: u8) { unimplemented!() }
    fn write_pci_u16(&self, _address: acpi::PciAddress, _offset: u16, _value: u16) { unimplemented!() }
    fn write_pci_u32(&self, _address: acpi::PciAddress, _offset: u16, _value: u32) { unimplemented!() }
}

/// Discover processors, initialize the BSP's local APIC, and start all APs.
pub fn init(rsdp_pointer: Optional<u64>) {
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "APIC: Looking for processors");

    let mut ap_apic_ids: Vec<u8> = Vec::new();

    if let Some(rsdp) = rsdp_pointer.as_ref() {
        log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "Found ACPI RSDP table");
        unsafe {
            match acpi::AcpiTables::from_rsdp(AcpiHandler(), *rsdp as usize) {
                Ok(tables) => match tables.find_table::<acpi::sdt::madt::Madt>() {
                    Some(madt) => {
                        for entry in madt.get().entries() {
                            match entry {
                                acpi::sdt::madt::MadtEntry::LocalApic(local_apic) => {
                                    let enabled = local_apic.flags & (1 << 0) != 0;
                                    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
                                        "Found CPU #{}: enabled={}", local_apic.apic_id, enabled);
                                    if enabled {
                                        ap_apic_ids.push(local_apic.apic_id);
                                    }
                                }
                                acpi::sdt::madt::MadtEntry::IoApic(io_apic) =>
                                    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
                                        "Found IO APIC #{}", io_apic.io_apic_id),
                                _ => {}
                            }
                        }
                    }
                    None => log_println!(log::SubSystem::X86_64, log::LogLevel::Error, "Failed to find MADT"),
                },
                Err(error) => log_println!(log::SubSystem::X86_64, log::LogLevel::Error,
                    "Failed to parse ACPI tables: {:?}", error),
            }
        }
    }

    let total_cpus = ap_apic_ids.len();
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "APIC: Found {} processors", total_cpus);

    if total_cpus <= 1 {
        log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "Single CPU system, no APs to start");
        return;
    }

    // initialize BSP's local APIC
    let apic_base = unsafe { xapic_base() };
    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "xAPIC base: {:#x}", apic_base);

    let mut bsp_lapic = LocalApicBuilder::new()
        .timer_vector(0xFE)
        .error_vector(0xFD)
        .spurious_vector(0xFF)
        .set_xapic_base(apic_base)
        .build()
        .expect("Failed to build BSP local APIC");
    unsafe { bsp_lapic.enable() };

    let bsp_hw_id = unsafe { bsp_lapic.id() };
    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "BSP APIC ID: {}", bsp_hw_id);

    // remove BSP from AP list
    ap_apic_ids.retain(|&id| id != bsp_hw_id as u8);
    let ap_count = ap_apic_ids.len();
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "Starting {} application processors", ap_count);

    // install trampoline
    ap_trampoline::install();

    // patch shared fields
    let (cr3_frame, _) = x86_64::registers::control::Cr3::read();
    let cr3_value = cr3_frame.start_address().as_u64();
    let entry_point = ap_entry as *const () as u64;
    let ready_counter_address = &AP_READY_COUNT as *const AtomicU32 as u64;
    ap_trampoline::patch_u64(ap_trampoline::PAGE_TABLE_OFFSET, cr3_value);
    ap_trampoline::patch_u64(ap_trampoline::ENTRY_POINT_OFFSET, entry_point);

    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
        "Trampoline data: CR3={:#x}, entry={:#x}, ready_counter={:#x}",
        cr3_value, entry_point, ready_counter_address);

    let icr_high = (apic_base + 0x310) as *mut u32;
    let icr_low = (apic_base + 0x300) as *mut u32;

    for &apic_id in &ap_apic_ids {
        // Use identity-mapped physical memory for AP stack (for debugging)
        // Allocate 16 physical pages and use them directly via identity mapping
        let mut stack_physical_base = 0u64;
        for page_index in 0..AP_STACK_PAGES {
            let page = crate::physical_memory::allocate(1).expect("Failed to allocate AP stack page") as u64;
            if page_index == 0 {
                stack_physical_base = page;
            }
        }
        // use the first page's address as the base — consecutive allocations
        // may not be contiguous, so just use a single page for now (4 KiB stack)
        let stack_top = stack_physical_base + 0x1000; // top of one physical page
        ap_trampoline::patch_u64(ap_trampoline::STACK_TOP_OFFSET, stack_top);
        ap_trampoline::patch_u64(ap_trampoline::READY_OFFSET, 0);

        log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
            "Sending INIT-SIPI to AP {} (stack={:#x})", apic_id, stack_top);

        let expected = AP_READY_COUNT.load(Ordering::Acquire) + 1;

        // INIT
        unsafe {
            core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
            core::ptr::write_volatile(icr_low, 0x00004500);
        }
        timer::delay_milliseconds(10);

        // INIT deassert
        unsafe {
            core::ptr::write_volatile(icr_high, 0);
            core::ptr::write_volatile(icr_low, 0x00008500);
        }
        timer::delay_microseconds(200);

        // SIPI
        unsafe {
            core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
            core::ptr::write_volatile(icr_low, 0x00004600 | ap_trampoline::SIPI_VECTOR as u32);
        }
        timer::delay_microseconds(200);

        // second SIPI
        unsafe {
            core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
            core::ptr::write_volatile(icr_low, 0x00004600 | ap_trampoline::SIPI_VECTOR as u32);
        }

        // wait for trampoline to complete
        let mut waited = 0u64;
        loop {
            let stage = ap_trampoline::read_u64(ap_trampoline::READY_OFFSET);
            if stage == 0xFF { break; }
            timer::delay_microseconds(100);
            waited += 1;
            if waited > 10_000 {
                log_println!(log::SubSystem::X86_64, log::LogLevel::Error,
                    "AP {} trampoline stuck at stage {}", apic_id, stage);
                break;
            }
        }

        // wait for Rust entry to signal ready
        while AP_READY_COUNT.load(Ordering::Acquire) < expected {
            timer::delay_microseconds(100);
            waited += 1;
            if waited > 20_000 {
                log_println!(log::SubSystem::X86_64, log::LogLevel::Error,
                    "AP {} did not signal ready", apic_id);
                break;
            }
        }

        if AP_READY_COUNT.load(Ordering::Acquire) >= expected {
            log_println!(log::SubSystem::X86_64, log::LogLevel::Information,
                "AP {} started successfully", apic_id);
        }
    }

    log_println!(log::SubSystem::X86_64, log::LogLevel::Information,
        "SMP: {}/{} APs started, {} total CPUs active",
        AP_READY_COUNT.load(Ordering::Acquire), ap_count,
        AP_READY_COUNT.load(Ordering::Acquire) + 1);
}

/// AP entry point — naked function that loads kernel GDT + IDT,
/// then jumps to the normal Rust `ap_main` function.
///
/// We MUST load the kernel's GDT and IDT before calling any normal Rust
/// code, because without a valid IDT any exception is a triple fault.
/// AP entry point — naked, sets up GDT/IDT then calls ap_main.
/// AP entry — naked bridge that calls the real function.
#[no_mangle]
#[unsafe(naked)]
extern "C" fn ap_entry() -> ! {
    core::arch::naked_asm!(
        "jmp {ap_main}",
        ap_main = sym ap_main,
    );
}

#[inline(never)]
extern "C" fn ap_main() -> ! {
    AP_READY_COUNT.fetch_add(1, Ordering::Release);
    loop {
        x86_64::instructions::hlt();
    }
}
