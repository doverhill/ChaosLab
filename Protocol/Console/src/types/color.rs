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
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Color, 1);
        pointer = pointer.offset(mem::size_of::<Color>() as isize);

        mem::size_of::<Color>() + self.write_references_at(pointer)
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
        mem::size_of::<Color>() + Self::reconstruct_at(object_pointer as *mut Color, object_pointer.offset(mem::size_of::<Color>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Color, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U8 alpha

        // U8 red

        // U8 green

        // U8 blue

        size
    }
}



