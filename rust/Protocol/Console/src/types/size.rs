#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Size {
    pub width: u64,
    pub height: u64,
}
impl Size {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, width: u64, height: u64) -> usize {
        let object: *mut Size = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Size>() as isize);

        // width
        (*object).width = width;

        // height
        (*object).height = height;

        // return
        mem::size_of::<Size>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut Size = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Size>() as isize);

        // width

        // height

        // return
        (mem::size_of::<Size>(), object.as_mut().unwrap())
    }
}


