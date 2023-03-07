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
pub struct HorizontalLayout {
    pub component_id: u64,
    pub parent_component_id: u64,
}

impl HorizontalLayout {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut HorizontalLayout, 1);
        pointer = pointer.offset(mem::size_of::<HorizontalLayout>() as isize);

        mem::size_of::<HorizontalLayout>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<HorizontalLayout>() + Self::reconstruct_at(object_pointer as *mut HorizontalLayout, object_pointer.offset(mem::size_of::<HorizontalLayout>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut HorizontalLayout, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        size
    }
}



