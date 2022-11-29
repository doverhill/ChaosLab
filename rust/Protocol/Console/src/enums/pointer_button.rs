#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(C, u64)]
pub enum PointerButtonEnum {
    Left,
    Right,
    Middle,
}

#[repr(C)]
struct PointerButtonEnumStruct {
    tag: PointerButtonStructTag,
    payload: PointerButtonStructPayload,
}

#[repr(u64)]
enum PointerButtonEnumStructTag {
    Left,
    Right,
    Middle,
}

#[repr(C)]
union PointerButtonEnumStructPayload {
    payload_left: [u8; 0],
    payload_right: [u8; 0],
    payload_middle: [u8; 0],
}

impl PointerButtonEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut PointerButtonEnum, 1);
        pointer = pointer.offset(mem::size_of::<PointerButtonEnum>() as isize);
        mem::size_of::<PointerButtonEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            PointerButtonEnum::Left => {
                0
            },
            PointerButtonEnum::Right => {
                0
            },
            PointerButtonEnum::Middle => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut PointerButtonEnum, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut PointerButtonEnumStruct;
        match ((*object).tag) {
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


