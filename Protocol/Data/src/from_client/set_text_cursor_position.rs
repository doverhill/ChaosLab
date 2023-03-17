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

pub struct SetTextCursorPositionParameters {
    pub position: TextPosition,
}

impl SetTextCursorPositionParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SetTextCursorPositionParameters, 1);
        pointer = pointer.offset(mem::size_of::<SetTextCursorPositionParameters>() as isize);

        mem::size_of::<SetTextCursorPositionParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType position
        let len = self.position.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<SetTextCursorPositionParameters>() + Self::reconstruct_at(object_pointer as *mut SetTextCursorPositionParameters, object_pointer.offset(mem::size_of::<SetTextCursorPositionParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SetTextCursorPositionParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType position
        let len = TextPosition::reconstruct_at(addr_of_mut!((*object_pointer).position), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}



