#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct UnwatchObjectParameters {
    pub watch_id: u64,
}

impl UnwatchObjectParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, watch_id: u64) -> usize {
        let object: *mut UnwatchObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<UnwatchObjectParameters>() as isize);

        // watch_id
        (*object).watch_id = watch_id;

        // return
        mem::size_of::<UnwatchObjectParameters>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut UnwatchObjectParameters, 1);
        let pointer = pointer.offset(mem::size_of::<UnwatchObjectParameters>() as isize);

        // watch_id

        // return
        mem::size_of::<UnwatchObjectParameters>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut UnwatchObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<UnwatchObjectParameters>() as isize);

        // watch_id

        // return
        (mem::size_of::<UnwatchObjectParameters>(), object)
    }
}


