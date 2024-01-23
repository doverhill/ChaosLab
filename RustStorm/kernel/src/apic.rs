use acpi::madt::MadtEntry::*;
use bootloader_api::info::Optional;
// use x2apic::{lapic::LocalApic, *};

use crate::serial_println;

#[derive(Clone)]
struct Handler();

impl acpi::AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping::new(physical_address, core::ptr::NonNull::<T>::new(physical_address as *mut T).unwrap(), size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}

pub fn init(rsdp_pointer: Optional<u64>) {
    serial_println!("Looking for processors");

    // use ACPI to find all processors
    let mut found_bsp = false;
    if let Some(rsdp) = rsdp_pointer.as_ref() {
        serial_println!("Found ACPI RSDP table");
        unsafe {
            match acpi::AcpiTables::from_rsdp(Handler(), *rsdp as usize) {
                Ok(tables) => match tables.find_table::<acpi::madt::Madt>() {
                    Ok(madt) => {
                        for entry in madt.entries() {
                            match entry {
                                LocalApic(local_apic) => {
                                    let enabled = local_apic.flags & (1 << 0) != 0;
                                    serial_println!("Found CPU #{}: enabled={}", local_apic.apic_id, enabled);
                                    if enabled {
                                        if !found_bsp {
                                            serial_println!("Found BSP CPU. Initializing Local APIC");
                                            found_bsp = true;
                                        }
                                        else {
                                            serial_println!("Starting CPU #{}", local_apic.apic_id);
                                        }
                                    }
                                }
                                IoApic(io_apic) => serial_println!("Found IO APIC #{}", io_apic.io_apic_id),
                                LocalApicAddressOverride(address) => {
                                    let local_apic_address = address.local_apic_address;
                                    serial_println!("Found 64 bit Local APIC address: {}", local_apic_address)
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(error) => {
                        serial_println!("Failed to find MADT: {:?}", error);
                    }
                },
                Err(error) => {
                    serial_println!("Failed to parse ACPI tables: {:?}", error)
                }
            }
        }
    }
}

// unsafe {
//     match lapic::LocalApicBuilder::new().set_xapic_base(lapic::xapic_base()).build() {
//         Ok(apic) => serial_println!("is bsp: {}", apic.is_bsp()),
//         Err(error) => serial_println!("failed to initalize lapic: {}", error),
//     }
// }
