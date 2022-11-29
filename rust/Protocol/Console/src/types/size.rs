#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

pub struct Size {
    pub width: u64,
    pub height: u64,
}

impl Size {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Size, 1);
        pointer = pointer.offset(mem::size_of::<Size>() as isize);

        mem::size_of::<Size>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 width

        // U64 height

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Size>() + Self::reconstruct_at(object_pointer as *mut Size, object_pointer.offset(mem::size_of::<Size>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Size, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 width

        // U64 height

        size
    }
}



