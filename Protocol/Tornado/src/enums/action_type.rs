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
pub enum ActionType {
    Optional,
    PrimarySafe,
    Dangerous,
}

#[repr(C)]
struct ActionTypeStruct {
    tag: ActionTypeStructTag,
    payload: ActionTypeStructPayload,
}

#[repr(u64)]
enum ActionTypeStructTag {
    Optional,
    PrimarySafe,
    Dangerous,
}

#[repr(C)]
union ActionTypeStructPayload {
    payload_optional: [u8; 0],
    payload_primary_safe: [u8; 0],
    payload_dangerous: [u8; 0],
}

impl ActionType {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ActionType, 1);
        pointer = pointer.offset(mem::size_of::<ActionType>() as isize);
        mem::size_of::<ActionType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            ActionType::Optional => {
                0
            },
            ActionType::PrimarySafe => {
                0
            },
            ActionType::Dangerous => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ActionType, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut ActionTypeStruct;
        match (*object).tag {
            ActionTypeStructTag::Optional => {
                0
            },
            ActionTypeStructTag::PrimarySafe => {
                0
            },
            ActionTypeStructTag::Dangerous => {
                0
            },
        }
    }
}



