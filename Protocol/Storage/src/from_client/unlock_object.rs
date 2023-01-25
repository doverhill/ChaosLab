#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

pub struct UnlockObjectParameters {
    pub lock_id: u64,
}

impl UnlockObjectParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut UnlockObjectParameters, 1);
        pointer = pointer.offset(mem::size_of::<UnlockObjectParameters>() as isize);

        mem::size_of::<UnlockObjectParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 lock_id

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<UnlockObjectParameters>() + Self::reconstruct_at(object_pointer as *mut UnlockObjectParameters, object_pointer.offset(mem::size_of::<UnlockObjectParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut UnlockObjectParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 lock_id

        size
    }
}



