#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(u64)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
}

#[repr(C)]
struct PointerButtonStruct {
    tag: PointerButtonStructTag,
    payload: PointerButtonStructPayload,
}

#[repr(u64)]
enum PointerButtonStructTag {
    Left,
    Right,
    Middle,
}

#[repr(C)]
union PointerButtonStructPayload {
    payload_left: [u8; 0],
    payload_right: [u8; 0],
    payload_middle: [u8; 0],
}

impl PointerButton {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut PointerButton, 1);
        pointer = pointer.offset(mem::size_of::<PointerButton>() as isize);
        mem::size_of::<PointerButton>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            PointerButton::Left => {
                0
            },
            PointerButton::Right => {
                0
            },
            PointerButton::Middle => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut PointerButton, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut PointerButtonStruct;
        match (*object).tag {
            PointerButtonStructTag::Left => {
                0
            },
            PointerButtonStructTag::Right => {
                0
            },
            PointerButtonStructTag::Middle => {
                0
            },
        }
    }
}



