#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct UnlockObjectParameters {
    pub lock_id: u64,
}

impl UnlockObjectParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, lock_id: u64) -> usize {
        let object: *mut UnlockObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<UnlockObjectParameters>() as isize);

        // lock_id
        (*object).lock_id = lock_id;

        // return
        mem::size_of::<UnlockObjectParameters>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut UnlockObjectParameters, 1);
        let pointer = pointer.offset(mem::size_of::<UnlockObjectParameters>() as isize);

        // lock_id

        // return
        mem::size_of::<UnlockObjectParameters>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut UnlockObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<UnlockObjectParameters>() as isize);

        // lock_id

        // return
        (mem::size_of::<UnlockObjectParameters>(), object)
    }
}


