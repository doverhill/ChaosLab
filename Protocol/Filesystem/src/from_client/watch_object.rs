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

pub struct WatchObjectParameters {
    pub object: Object,
}

impl WatchObjectParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut WatchObjectParameters, 1);
        pointer = pointer.offset(mem::size_of::<WatchObjectParameters>() as isize);

        mem::size_of::<WatchObjectParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType object
        let len = self.object.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<WatchObjectParameters>() + Self::reconstruct_at(object_pointer as *mut WatchObjectParameters, object_pointer.offset(mem::size_of::<WatchObjectParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut WatchObjectParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType object
        let len = Object::reconstruct_at(addr_of_mut!((*object_pointer).object), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}

use alloc::vec::Vec;
use alloc::string::String;

#[derive(Copy, Clone)]
pub struct WatchObjectReturns {
    pub watch_id: u64,
}

impl WatchObjectReturns {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut WatchObjectReturns, 1);
        pointer = pointer.offset(mem::size_of::<WatchObjectReturns>() as isize);

        mem::size_of::<WatchObjectReturns>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 watch_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<WatchObjectReturns>() + Self::reconstruct_at(object_pointer as *mut WatchObjectReturns, object_pointer.offset(mem::size_of::<WatchObjectReturns>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut WatchObjectReturns, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 watch_id

        size
    }
}



