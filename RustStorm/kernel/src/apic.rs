use acpi::sdt::madt::{LocalApicEntry, MadtEntry::*};
use alloc::vec::Vec;
use bootloader_api::info::Optional;

use crate::{log, log_println};

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

    fn read_pci_u8(&self, _address: acpi::PciAddress, _offset: u16) -> u8 { unimplemented!("PCI config read") }
    fn read_pci_u16(&self, _address: acpi::PciAddress, _offset: u16) -> u16 { unimplemented!("PCI config read") }
    fn read_pci_u32(&self, _address: acpi::PciAddress, _offset: u16) -> u32 { unimplemented!("PCI config read") }

    fn write_pci_u8(&self, _address: acpi::PciAddress, _offset: u16, _value: u8) { unimplemented!("PCI config write") }
    fn write_pci_u16(&self, _address: acpi::PciAddress, _offset: u16, _value: u16) { unimplemented!("PCI config write") }
    fn write_pci_u32(&self, _address: acpi::PciAddress, _offset: u16, _value: u32) { unimplemented!("PCI config write") }
}

struct Processor {
    pub apic: LocalApicEntry
}

impl Processor {
    pub fn new(apic: LocalApicEntry) -> Self {
        Self { apic }
    }
}

pub fn init(rsdp_pointer: Optional<u64>) {
    log_println!(log::SubSystem::X86_64, log::LogLevel::Information, "APIC: Looking for processors");

    let mut processors: Vec<Processor> = Vec::new();

    // use ACPI to find all processors
    // let mut found_bsp = false;
    if let Some(rsdp) = rsdp_pointer.as_ref() {
        log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "Found ACPI RSDP table");
        unsafe {
            match acpi::AcpiTables::from_rsdp(AcpiHandler(), *rsdp as usize) {
                Ok(tables) => match tables.find_table::<acpi::sdt::madt::Madt>() {
                    Some(madt) => {
                        for entry in madt.get().entries() {
                            match entry {
                                LocalApic(local_apic) => {
                                    let enabled = local_apic.flags & (1 << 0) != 0;
                                    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "Found CPU #{}: enabled={}", local_apic.apic_id, enabled);
                                    if enabled {
                                        processors.push(Processor::new(*local_apic));
                                    }
                                }
                                IoApic(io_apic) => log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "Found IO APIC #{}", io_apic.io_apic_id),
                                LocalApicAddressOverride(address) => {
                                    let local_apic_address = address.local_apic_address;
                                    log_println!(log::SubSystem::X86_64, log::LogLevel::Debug, "Found 64 bit Local APIC address: {}", local_apic_address)
                                }
                                _ => {}
                            }
                        }
                    }
                    None => {
                        log_println!(log::SubSystem::X86_64, log::LogLevel::Error, "Failed to find MADT");
                    }
                },
                Err(error) => {
                    log_println!(log::SubSystem::X86_64, log::LogLevel::Error, "Failed to parse ACPI tables: {:?}", error)
                }
            }
        }
    }

    // we are running on BSP, initalize local APIC so that we can send IPI to other processors
    for p in processors {
        log_println!(log::SubSystem::X86_64, log::LogLevel::Error, "Intializing CPU #{}", p.apic.apic_id);

    }

}

// unsafe {
//     match lapic::LocalApicBuilder::new().set_xapic_base(lapic::xapic_base()).build() {
//         Ok(apic) => serial_println!("is bsp: {}", apic.is_bsp()),
//         Err(error) => serial_println!("failed to initalize lapic: {}", error),
//     }
// }
