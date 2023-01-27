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

pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Point, 1);
        pointer = pointer.offset(mem::size_of::<Point>() as isize);

        mem::size_of::<Point>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // I64 x

        // I64 y

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Point>() + Self::reconstruct_at(object_pointer as *mut Point, object_pointer.offset(mem::size_of::<Point>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Point, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // I64 x

        // I64 y

        size
    }
}



