#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(u64)]
pub enum SizeMode {
    ContentMinimum,
    ContentMaximum,
    Fraction,
}

#[repr(C)]
struct SizeModeStruct {
    tag: SizeModeStructTag,
    payload: SizeModeStructPayload,
}

#[repr(u64)]
enum SizeModeStructTag {
    ContentMinimum,
    ContentMaximum,
    Fraction,
}

#[repr(C)]
union SizeModeStructPayload {
    payload_content_minimum: [u8; 0],
    payload_content_maximum: [u8; 0],
    payload_fraction: [u8; 0],
}

impl SizeMode {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SizeMode, 1);
        pointer = pointer.offset(mem::size_of::<SizeMode>() as isize);
        mem::size_of::<SizeMode>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            SizeMode::ContentMinimum => {
                0
            },
            SizeMode::ContentMaximum => {
                0
            },
            SizeMode::Fraction => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SizeMode, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut SizeModeStruct;
        match (*object).tag {
            SizeModeStructTag::ContentMinimum => {
                0
            },
            SizeModeStructTag::ContentMaximum => {
                0
            },
            SizeModeStructTag::Fraction => {
                0
            },
        }
    }
}



