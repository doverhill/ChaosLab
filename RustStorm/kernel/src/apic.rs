//! APIC initialization and SMP startup.
//!
//! Discovers processors via ACPI MADT, initializes the BSP's local APIC,
//! then starts all Application Processors (APs) using the INIT-SIPI-SIPI
//! protocol with a real-mode trampoline.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use bootloader_api::info::Optional;
use x2apic::lapic::{LocalApic, LocalApicBuilder, xapic_base};

use crate::ap_trampoline;
use crate::virtual_memory;
use crate::{log, log_println, qemu};

/// Number of APs that have completed initialization and are ready.
static AP_READY_COUNT: AtomicU32 = AtomicU32::new(0);

/// Per-AP stack size (64 KiB).
const AP_STACK_PAGES: usize = 16;

/// ACPI handler for accessing physical memory via identity mapping.
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

    // discover processors via ACPI MADT
    let mut ap_apic_ids: Vec<u8> = Vec::new();
    let mut bsp_apic_id: Option<u8> = None;

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
    let mut bsp_lapic = unsafe {
        LocalApicBuilder::new()
            .timer_vector(0xFE)
            .error_vector(0xFD)
            .spurious_vector(0xFF)
            .set_xapic_base(xapic_base())
            .build()
            .expect("Failed to build BSP local APIC")
    };
    unsafe { bsp_lapic.enable() };

    // the BSP is the first APIC ID in the list (APIC ID from RDMSR)
    bsp_apic_id = Some(ap_apic_ids[0]);
    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "BSP APIC ID: {}", ap_apic_ids[0]);

    // remove BSP from the AP list
    let bsp_id = bsp_apic_id.unwrap();
    ap_apic_ids.retain(|&id| id != bsp_id);
    let ap_count = ap_apic_ids.len();
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "Starting {} application processors", ap_count);

    // install the trampoline at physical 0x8000
    ap_trampoline::install();

    // patch fields shared across all APs
    let (cr3_frame, _) = x86_64::registers::control::Cr3::read();
    let cr3_value = cr3_frame.start_address().as_u64();
    ap_trampoline::patch_u64(ap_trampoline::PAGE_TABLE_OFFSET, cr3_value);
    ap_trampoline::patch_u64(ap_trampoline::ENTRY_POINT_OFFSET, ap_entry as *const () as u64);

    // start each AP one at a time (reuse the same trampoline page)
    for &apic_id in &ap_apic_ids {
        // allocate a unique stack for this AP
        let stack_base = virtual_memory::allocate_contiguous_pages(AP_STACK_PAGES)
            .expect("Failed to allocate AP stack");
        let stack_top = stack_base as u64 + (AP_STACK_PAGES * 0x1000) as u64;
        ap_trampoline::patch_u64(ap_trampoline::STACK_TOP_OFFSET, stack_top);

        // clear the ready flag before starting this AP
        ap_trampoline::patch_u64(ap_trampoline::READY_OFFSET, 0);

        log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
            "Sending INIT-SIPI to AP {} (stack top={:#x})", apic_id, stack_top);

        let expected = AP_READY_COUNT.load(Ordering::Acquire) + 1;

        // INIT IPI
        unsafe { bsp_lapic.send_init_ipi(apic_id as u32) };
        qemu::delay_milliseconds(10);

        // single SIPI (works reliably on modern hardware and QEMU)
        unsafe { bsp_lapic.send_sipi(ap_trampoline::SIPI_VECTOR, apic_id as u32) };

        // wait for AP to signal trampoline is done (ready flag at 0x8008)
        let mut waited_milliseconds = 0u64;
        while ap_trampoline::read_u64(ap_trampoline::READY_OFFSET) == 0 {
            qemu::delay_microseconds(100);
            waited_milliseconds += 1;
            if waited_milliseconds > 10_000 { // ~1 second (100us * 10000)
                log_println!(log::SubSystem::X86_64, log::LogLevel::Error,
                    "AP {} trampoline did not complete within 1 second", apic_id);
                break;
            }
        }

        // wait for AP to finish Rust init (AP_READY_COUNT incremented)
        while AP_READY_COUNT.load(Ordering::Acquire) < expected {
            qemu::delay_microseconds(100);
            waited_milliseconds += 1;
            if waited_milliseconds > 20_000 {
                log_println!(log::SubSystem::X86_64, log::LogLevel::Error,
                    "AP {} Rust init did not complete within 2 seconds", apic_id);
                break;
            }
        }

        if AP_READY_COUNT.load(Ordering::Acquire) >= expected {
            log_println!(log::SubSystem::X86_64, log::LogLevel::Debug,
                "AP {} started successfully ({}ms)", apic_id, waited_milliseconds);
        }
    }

    log_println!(log::SubSystem::X86_64, log::LogLevel::Information,
        "SMP: {}/{} APs started, {} total CPUs active",
        AP_READY_COUNT.load(Ordering::Acquire), ap_count, AP_READY_COUNT.load(Ordering::Acquire) + 1);
}

// ---------------------------------------------------------------------------
// AP entry point — called by the trampoline after switching to long mode
// ---------------------------------------------------------------------------

/// Entry point for Application Processors after the trampoline has set up
/// long mode, loaded the page tables, and set the stack pointer.
extern "C" fn ap_entry() -> ! {
    // we're now in 64-bit mode with identity-mapped memory and a valid stack

    // reload the kernel's GDT and IDT (the trampoline used a temporary GDT)
    crate::gdt::init();
    crate::interrupts::init_exceptions();

    // TODO: initialize this AP's local APIC

    // signal that this AP is ready
    let cpu_number = AP_READY_COUNT.fetch_add(1, Ordering::Release) + 1;

    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "Hello from CPU {}", cpu_number);

    // idle loop
    loop {
        x86_64::instructions::hlt();
    }
}
