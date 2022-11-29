#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

pub struct DrawImagePatchParameters {
    pub image_patch: ImagePatch,
}

impl DrawImagePatchParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut DrawImagePatchParameters, 1);
        pointer = pointer.offset(mem::size_of::<DrawImagePatchParameters>() as isize);

        mem::size_of::<DrawImagePatchParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType image_patch
        let len = self.image_patch.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<DrawImagePatchParameters>() + Self::reconstruct_at(object_pointer as *mut DrawImagePatchParameters, object_pointer.offset(mem::size_of::<DrawImagePatchParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut DrawImagePatchParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType image_patch
        let len = ImagePatch::reconstruct_at(addr_of_mut!((*object_pointer).image_patch), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}



