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

pub struct SetTextColorParameters {
    pub foreground: Color,
    pub background: Color,
}

impl SetTextColorParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SetTextColorParameters, 1);
        pointer = pointer.offset(mem::size_of::<SetTextColorParameters>() as isize);

        mem::size_of::<SetTextColorParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType foreground
        let len = self.foreground.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType background
        let len = self.background.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<SetTextColorParameters>() + Self::reconstruct_at(object_pointer as *mut SetTextColorParameters, object_pointer.offset(mem::size_of::<SetTextColorParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SetTextColorParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType foreground
        let len = Color::reconstruct_at(addr_of_mut!((*object_pointer).foreground), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType background
        let len = Color::reconstruct_at(addr_of_mut!((*object_pointer).background), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}



