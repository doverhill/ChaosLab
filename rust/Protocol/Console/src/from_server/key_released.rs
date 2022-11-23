#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct KeyReleasedParameters {
    pub key_code: KeyCode,
}
impl KeyReleasedParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, key_code: KeyCode) -> usize {
        let object: *mut KeyReleasedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyReleasedParameters>() as isize);

        // key_code

        // return
        mem::size_of::<KeyReleasedParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut KeyReleasedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyReleasedParameters>() as isize);

        // key_code

        // return
        (mem::size_of::<KeyReleasedParameters>(), object)
    }
}


