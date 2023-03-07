#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use alloc::vec::Vec;
use alloc::string::String;
use crate::types::*;
use crate::enums::*;

#[repr(u64)]
pub enum Severity {
    Normal,
    Information,
    Warning,
    Error,
}

#[repr(C)]
struct SeverityStruct {
    tag: SeverityStructTag,
    payload: SeverityStructPayload,
}

#[repr(u64)]
enum SeverityStructTag {
    Normal,
    Information,
    Warning,
    Error,
}

#[repr(C)]
union SeverityStructPayload {
    payload_normal: [u8; 0],
    payload_information: [u8; 0],
    payload_warning: [u8; 0],
    payload_error: [u8; 0],
}

impl Severity {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Severity, 1);
        pointer = pointer.offset(mem::size_of::<Severity>() as isize);
        mem::size_of::<Severity>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            Severity::Normal => {
                0
            },
            Severity::Information => {
                0
            },
            Severity::Warning => {
                0
            },
            Severity::Error => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Severity, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut SeverityStruct;
        match (*object).tag {
            SeverityStructTag::Normal => {
                0
            },
            SeverityStructTag::Information => {
                0
            },
            SeverityStructTag::Warning => {
                0
            },
            SeverityStructTag::Error => {
                0
            },
        }
    }
}



