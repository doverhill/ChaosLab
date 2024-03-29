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
pub struct TextColor {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl TextColor {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut TextColor, 1);
        pointer = pointer.offset(mem::size_of::<TextColor>() as isize);

        mem::size_of::<TextColor>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U8 alpha

        // U8 red

        // U8 green

        // U8 blue

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<TextColor>() + Self::reconstruct_at(object_pointer as *mut TextColor, object_pointer.offset(mem::size_of::<TextColor>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut TextColor, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U8 alpha

        // U8 red

        // U8 green

        // U8 blue

        size
    }
}



