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

use alloc::vec::Vec;
use alloc::string::String;

#[derive(Copy, Clone)]
pub struct CharacterInputParameters {
    pub character: u64,
}

impl CharacterInputParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut CharacterInputParameters, 1);
        pointer = pointer.offset(mem::size_of::<CharacterInputParameters>() as isize);

        mem::size_of::<CharacterInputParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 character

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<CharacterInputParameters>() + Self::reconstruct_at(object_pointer as *mut CharacterInputParameters, object_pointer.offset(mem::size_of::<CharacterInputParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut CharacterInputParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 character

        size
    }
}



