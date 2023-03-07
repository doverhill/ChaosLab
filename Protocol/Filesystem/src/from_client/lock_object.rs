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

pub struct LockObjectParameters {
    pub object: Object,
}

impl LockObjectParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut LockObjectParameters, 1);
        pointer = pointer.offset(mem::size_of::<LockObjectParameters>() as isize);

        mem::size_of::<LockObjectParameters>() + self.write_references_at(pointer)
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
        mem::size_of::<LockObjectParameters>() + Self::reconstruct_at(object_pointer as *mut LockObjectParameters, object_pointer.offset(mem::size_of::<LockObjectParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut LockObjectParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType object
        let len = Object::reconstruct_at(addr_of_mut!((*object_pointer).object), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        size
    }
}

#[derive(Copy, Clone)]
pub struct LockObjectReturns {
    pub lock_id: u64,
}

impl LockObjectReturns {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut LockObjectReturns, 1);
        pointer = pointer.offset(mem::size_of::<LockObjectReturns>() as isize);

        mem::size_of::<LockObjectReturns>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 lock_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<LockObjectReturns>() + Self::reconstruct_at(object_pointer as *mut LockObjectReturns, object_pointer.offset(mem::size_of::<LockObjectReturns>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut LockObjectReturns, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 lock_id

        size
    }
}



