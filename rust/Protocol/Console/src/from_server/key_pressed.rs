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

pub struct KeyPressedParameters {
    pub key_code: KeyCode,
}

impl KeyPressedParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut KeyPressedParameters, 1);
        pointer = pointer.offset(mem::size_of::<KeyPressedParameters>() as isize);

        mem::size_of::<KeyPressedParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // Enum key_code

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<KeyPressedParameters>() + Self::reconstruct_at(object_pointer as *mut KeyPressedParameters, object_pointer.offset(mem::size_of::<KeyPressedParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut KeyPressedParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // Enum key_code

        size
    }
}



