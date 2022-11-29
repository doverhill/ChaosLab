#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(u64)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
}

#[repr(C)]
struct KeyCodeStruct {
    tag: KeyCodeStructTag,
    payload: KeyCodeStructPayload,
}

#[repr(u64)]
enum KeyCodeStructTag {
    A,
    B,
    C,
    D,
    E,
}

#[repr(C)]
union KeyCodeStructPayload {
    payload_a: [u8; 0],
    payload_b: [u8; 0],
    payload_c: [u8; 0],
    payload_d: [u8; 0],
    payload_e: [u8; 0],
}

impl KeyCode {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut KeyCode, 1);
        pointer = pointer.offset(mem::size_of::<KeyCode>() as isize);
        mem::size_of::<KeyCode>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            KeyCode::A => {
                0
            },
            KeyCode::B => {
                0
            },
            KeyCode::C => {
                0
            },
            KeyCode::D => {
                0
            },
            KeyCode::E => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut KeyCode, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut KeyCodeStruct;
        match (*object).tag {
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



