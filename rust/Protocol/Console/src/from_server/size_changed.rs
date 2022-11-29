#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

pub struct SizeChangedParameters {
    pub framebuffer_size: Size,
    pub text_size: Size,
}

impl SizeChangedParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SizeChangedParameters, 1);
        pointer = pointer.offset(mem::size_of::<SizeChangedParameters>() as isize);

        mem::size_of::<SizeChangedParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType framebuffer_size
        let len = self.framebuffer_size.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType text_size
        let len = self.text_size.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<SizeChangedParameters>() + Self::reconstruct_at(object_pointer as *mut SizeChangedParameters, object_pointer.offset(mem::size_of::<SizeChangedParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SizeChangedParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType framebuffer_size
        let len = Size::reconstruct_at(addr_of_mut!((*object_pointer).framebuffer_size), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType text_size
        let len = Size::reconstruct_at(addr_of_mut!((*object_pointer).text_size), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}



