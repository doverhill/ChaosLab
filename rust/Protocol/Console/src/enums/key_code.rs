#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(C, u64)]
pub enum KeyCodeEnum {
    A,
    B,
    C,
    D,
    E,
}

#[repr(C)]
struct KeyCodeEnumStruct {
    tag: KeyCodeStructTag,
    payload: KeyCodeStructPayload,
}

#[repr(u64)]
enum KeyCodeEnumStructTag {
    A,
    B,
    C,
    D,
    E,
}

#[repr(C)]
union KeyCodeEnumStructPayload {
    payload_a: [u8; 0],
    payload_b: [u8; 0],
    payload_c: [u8; 0],
    payload_d: [u8; 0],
    payload_e: [u8; 0],
}

impl KeyCodeEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut KeyCodeEnum, 1);
        pointer = pointer.offset(mem::size_of::<KeyCodeEnum>() as isize);
        mem::size_of::<KeyCodeEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            KeyCodeEnum::A => {
                0
            },
            KeyCodeEnum::B => {
                0
            },
            KeyCodeEnum::C => {
                0
            },
            KeyCodeEnum::D => {
                0
            },
            KeyCodeEnum::E => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut KeyCodeEnum, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut KeyCodeEnumStruct;
        match ((*object).tag) {
            KeyCodeStructTag::A => {
                0
            },
            KeyCodeStructTag::B => {
                0
            },
            KeyCodeStructTag::C => {
                0
            },
            KeyCodeStructTag::D => {
                0
            },
            KeyCodeStructTag::E => {
                0
            },
        }
    }
}


