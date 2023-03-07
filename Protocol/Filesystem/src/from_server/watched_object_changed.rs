#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

use alloc::vec::Vec;
use alloc::string::String;

#[derive(Copy, Clone)]
pub struct WatchedObjectChangedParameters {
    pub watch_id: u64,
}

impl WatchedObjectChangedParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut WatchedObjectChangedParameters, 1);
        pointer = pointer.offset(mem::size_of::<WatchedObjectChangedParameters>() as isize);

        mem::size_of::<WatchedObjectChangedParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 watch_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<WatchedObjectChangedParameters>() + Self::reconstruct_at(object_pointer as *mut WatchedObjectChangedParameters, object_pointer.offset(mem::size_of::<WatchedObjectChangedParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut WatchedObjectChangedParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 watch_id

        size
    }
}



