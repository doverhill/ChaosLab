//! Process management and ELF loading.
//!
//! Each process owns an [`AddressSpace`] and is created from an ELF binary.
//! `create_from_elf` validates the image, maps LOAD segments into a fresh
//! user address space, allocates a user stack, and returns a process ready
//! to be launched via SYSRETQ.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::address_space::AddressSpace;
use crate::page_tables::{PAGE_SIZE, USER_VIRTUAL_BASE};
use crate::{log, log_println};

static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);

/// Where we load the ELF image within user virtual space.
/// The ELF's virtual addresses are offset by this base.
const ELF_LOAD_BASE: u64 = USER_VIRTUAL_BASE;

/// User stack: 16 pages (64 KiB), placed 16 MiB above the load base.
const USER_STACK_PAGES: usize = 16;
const USER_STACK_OFFSET: u64 = 16 * 1024 * 1024; // 16 MiB from base

/// A process ready to be launched.
pub struct Process {
    pub process_id: u64,
    pub address_space: AddressSpace,
    pub entry_point: u64,
    pub user_stack_top: u64,
}

impl Process {
    /// Create a process from a raw ELF image.
    ///
    /// Validates the ELF headers, creates a user address space, maps each
    /// PT_LOAD segment, copies segment data, and allocates a user stack.
    ///
    /// The caller's `elf_data` slice can be freed after this returns — all
    /// needed data has been copied into the new address space's physical pages.
    ///
    /// Returns Err with a human-readable message on any validation failure.
    pub fn create_from_elf(elf_data: &[u8]) -> Result<Self, &'static str> {
        // --- ELF header validation ---
        if elf_data.len() < 64 {
            return Err("ELF image too small for header");
        }

        // magic
        if &elf_data[0..4] != b"\x7fELF" {
            return Err("invalid ELF magic");
        }

        // class: must be 64-bit (2)
        if elf_data[4] != 2 {
            return Err("not a 64-bit ELF");
        }

        // endian: must be little-endian (1)
        if elf_data[5] != 1 {
            return Err("not a little-endian ELF");
        }

        // machine: must be x86_64 (0x3E)
        let machine = u16::from_le_bytes([elf_data[18], elf_data[19]]);
        if machine != 0x3E {
            return Err("not an x86_64 ELF");
        }

        // entry point
        let entry_point_offset = u64::from_le_bytes(elf_data[24..32].try_into().unwrap());

        // program header table offset and entry count
        let program_header_offset = u64::from_le_bytes(elf_data[32..40].try_into().unwrap()) as usize;
        let program_header_entry_size = u16::from_le_bytes([elf_data[54], elf_data[55]]) as usize;
        let program_header_count = u16::from_le_bytes([elf_data[56], elf_data[57]]) as usize;

        if program_header_entry_size < 56 {
            return Err("program header entry too small");
        }

        let program_header_end = program_header_offset
            .checked_add(program_header_count.checked_mul(program_header_entry_size).ok_or("program header table overflow")?)
            .ok_or("program header table overflow")?;
        if program_header_end > elf_data.len() {
            return Err("program header table extends beyond ELF data");
        }

        log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
            "ELF: entry={:#x}, {} program headers", entry_point_offset, program_header_count);

        // --- Create address space and map segments ---
        let address_space = AddressSpace::new();
        let base = ELF_LOAD_BASE;

        for index in 0..program_header_count {
            let header_start = program_header_offset + index * program_header_entry_size;
            let header = &elf_data[header_start..header_start + program_header_entry_size];

            let segment_type = u32::from_le_bytes(header[0..4].try_into().unwrap());

            // PT_LOAD = 1
            if segment_type != 1 {
                continue;
            }

            let file_offset = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;
            let virtual_address = u64::from_le_bytes(header[16..24].try_into().unwrap());
            let file_size = u64::from_le_bytes(header[32..40].try_into().unwrap()) as usize;
            let memory_size = u64::from_le_bytes(header[40..48].try_into().unwrap()) as usize;

            if file_size > memory_size {
                return Err("PT_LOAD file_size > memory_size");
            }

            // validate file data bounds
            let file_end = file_offset.checked_add(file_size).ok_or("segment file range overflow")?;
            if file_end > elf_data.len() {
                return Err("PT_LOAD segment extends beyond ELF data");
            }

            if memory_size == 0 {
                continue;
            }

            // compute the page-aligned virtual range we need to map
            let segment_virtual_start = base + virtual_address;
            let page_aligned_start = segment_virtual_start & !(PAGE_SIZE - 1);
            let page_aligned_end = (segment_virtual_start + memory_size as u64 + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            let page_count = ((page_aligned_end - page_aligned_start) / PAGE_SIZE) as usize;

            log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
                "ELF LOAD: vaddr={:#x} -> {:#x}, filesz={:#x}, memsz={:#x}, {} pages",
                segment_virtual_start, segment_virtual_start + memory_size as u64,
                file_size, memory_size, page_count);

            // map zero-filled pages at the target address
            let physical_addresses = address_space.map_user_pages_at(page_aligned_start, page_count)?;

            // copy file data into the mapped pages via identity-mapped physical addresses
            if file_size > 0 {
                let offset_within_first_page = (segment_virtual_start - page_aligned_start) as usize;
                let source = &elf_data[file_offset..file_offset + file_size];
                let mut bytes_copied = 0;

                for (page_index, &physical_address) in physical_addresses.iter().enumerate() {
                    let page_start = if page_index == 0 { offset_within_first_page } else { 0 };
                    let page_end = PAGE_SIZE as usize;
                    let available = page_end - page_start;
                    let remaining = file_size - bytes_copied;
                    let to_copy = available.min(remaining);

                    if to_copy > 0 {
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                source[bytes_copied..].as_ptr(),
                                (physical_address + page_start as u64) as *mut u8,
                                to_copy,
                            );
                        }
                        bytes_copied += to_copy;
                    }

                    if bytes_copied >= file_size {
                        break;
                    }
                }
            }
            // BSS portion (memory_size - file_size) is already zero from map_user_pages_at
        }

        // --- Process R_X86_64_RELATIVE relocations ---
        // Static PIE binaries have a .rela.dyn section with R_X86_64_RELATIVE
        // entries. Each one writes (base + addend) to (base + offset).
        // We find the RELA table via the PT_DYNAMIC segment.
        let mut rela_address: Option<u64> = None;
        let mut rela_size: usize = 0;
        let mut rela_entry_size: usize = 24; // default Elf64_Rela size

        // first pass: find PT_DYNAMIC (type 2) to get .rela.dyn info
        for index in 0..program_header_count {
            let header_start = program_header_offset + index * program_header_entry_size;
            let header = &elf_data[header_start..header_start + program_header_entry_size];
            let segment_type = u32::from_le_bytes(header[0..4].try_into().unwrap());

            // PT_DYNAMIC = 2
            if segment_type != 2 {
                continue;
            }

            let dyn_offset = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;
            let dyn_size = u64::from_le_bytes(header[32..40].try_into().unwrap()) as usize;

            if dyn_offset + dyn_size > elf_data.len() {
                return Err("PT_DYNAMIC extends beyond ELF data");
            }

            // parse dynamic entries (each 16 bytes: tag + value)
            let mut pos = dyn_offset;
            while pos + 16 <= dyn_offset + dyn_size {
                let tag = u64::from_le_bytes(elf_data[pos..pos + 8].try_into().unwrap());
                let value = u64::from_le_bytes(elf_data[pos + 8..pos + 16].try_into().unwrap());
                match tag {
                    0 => break,       // DT_NULL — end of dynamic section
                    7 => rela_address = Some(value),  // DT_RELA
                    8 => rela_size = value as usize,  // DT_RELASZ
                    9 => rela_entry_size = value as usize,  // DT_RELAENT
                    _ => {}
                }
                pos += 16;
            }
            break;
        }

        // apply relocations
        if let Some(rela_vaddr) = rela_address {
            if rela_entry_size < 24 {
                return Err("RELA entry size too small");
            }

            // rela_vaddr is a virtual address in the ELF (offset from 0)
            let rela_file_offset = rela_vaddr as usize;
            if rela_file_offset + rela_size > elf_data.len() {
                return Err("RELA table extends beyond ELF data");
            }

            let relocation_count = rela_size / rela_entry_size;
            let mut applied = 0u64;

            for index in 0..relocation_count {
                let entry_start = rela_file_offset + index * rela_entry_size;
                let offset = u64::from_le_bytes(elf_data[entry_start..entry_start + 8].try_into().unwrap());
                let info = u64::from_le_bytes(elf_data[entry_start + 8..entry_start + 16].try_into().unwrap());
                let addend = u64::from_le_bytes(elf_data[entry_start + 16..entry_start + 24].try_into().unwrap());

                let relocation_type = info & 0xFFFF_FFFF;

                // R_X86_64_RELATIVE = 8: *location = base + addend
                if relocation_type == 8 {
                    let target_virtual = base.wrapping_add(offset);
                    let value = base.wrapping_add(addend);

                    // translate target virtual address to physical for kernel write
                    let target_physical = address_space.virtual_to_physical_user(target_virtual)
                        .ok_or("relocation target not mapped")?;

                    unsafe {
                        core::ptr::write(target_physical as *mut u64, value);
                    }
                    applied += 1;
                }
            }

            log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
                "ELF: applied {} R_X86_64_RELATIVE relocations", applied);
        }

        // --- Allocate user stack ---
        let stack_base = base + USER_STACK_OFFSET;
        address_space.map_user_pages_at(stack_base, USER_STACK_PAGES)?;
        let user_stack_top = stack_base + (USER_STACK_PAGES as u64 * PAGE_SIZE);

        // entry point = base + ELF entry offset
        let entry_point = base + entry_point_offset;

        let process_id = NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed);

        log_println!(log::SubSystem::Kernel, log::LogLevel::Information,
            "Process {} created from ELF: entry={:#x}, stack={:#x}, L4={:#x}",
            process_id, entry_point, user_stack_top, address_space.l4_physical_address());

        Ok(Process {
            process_id,
            address_space,
            entry_point,
            user_stack_top,
        })
    }
}
