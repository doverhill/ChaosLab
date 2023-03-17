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

#[derive(Copy, Clone)]
pub struct TextSize {
    pub columns: u64,
    pub rows: u64,
}

impl TextSize {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut TextSize, 1);
        pointer = pointer.offset(mem::size_of::<TextSize>() as isize);

        mem::size_of::<TextSize>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 columns

        // U64 rows

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<TextSize>() + Self::reconstruct_at(object_pointer as *mut TextSize, object_pointer.offset(mem::size_of::<TextSize>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut TextSize, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 columns

        // U64 rows

        size
    }
}



