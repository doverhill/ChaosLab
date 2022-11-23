#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct KeyPressedParameters {
    pub key_code: KeyCode,
}
impl KeyPressedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, key_code: KeyCode) -> usize {
        let object: *mut KeyPressedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyPressedParameters>() as isize);

        // key_code

        // return
        mem::size_of::<KeyPressedParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut KeyPressedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyPressedParameters>() as isize);

        // key_code

        // return
        (mem::size_of::<KeyPressedParameters>(), object.as_mut().unwrap())
    }
}


