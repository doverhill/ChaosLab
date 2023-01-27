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

pub struct ImagePatch {
    pub image: Image,
    pub position: Point,
}

impl ImagePatch {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ImagePatch, 1);
        pointer = pointer.offset(mem::size_of::<ImagePatch>() as isize);

        mem::size_of::<ImagePatch>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType image
        let len = self.image.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType position
        let len = self.position.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ImagePatch>() + Self::reconstruct_at(object_pointer as *mut ImagePatch, object_pointer.offset(mem::size_of::<ImagePatch>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ImagePatch, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType image
        let len = Image::reconstruct_at(addr_of_mut!((*object_pointer).image), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType position
        let len = Point::reconstruct_at(addr_of_mut!((*object_pointer).position), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}



