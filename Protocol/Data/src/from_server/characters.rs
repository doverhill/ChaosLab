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

pub struct CharactersParameters {
    pub characters: Vec<u64>,
}

impl CharactersParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut CharactersParameters, 1);
        pointer = pointer.offset(mem::size_of::<CharactersParameters>() as isize);

        mem::size_of::<CharactersParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 characters
        let len = self.characters.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.characters.as_ptr(), pointer as *mut u64, len);
        pointer = pointer.offset(len as isize * mem::size_of::<u64>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<u64>();

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<CharactersParameters>() + Self::reconstruct_at(object_pointer as *mut CharactersParameters, object_pointer.offset(mem::size_of::<CharactersParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut CharactersParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 characters
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut u64, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).characters), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<u64>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<u64>() as isize);
        pointer = references_pointer;

        size
    }
}



