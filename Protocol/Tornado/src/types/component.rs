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
pub struct Component {
    pub component_id: u64,
    pub parent_component_id: u64,
}

impl Component {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Component, 1);
        pointer = pointer.offset(mem::size_of::<Component>() as isize);

        mem::size_of::<Component>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Component>() + Self::reconstruct_at(object_pointer as *mut Component, object_pointer.offset(mem::size_of::<Component>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Component, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        size
    }
}



