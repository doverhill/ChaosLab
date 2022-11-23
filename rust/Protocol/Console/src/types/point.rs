#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, x: i64, y: i64) -> usize {
        let object: *mut Point = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Point>() as isize);

        // x
        (*object).x = x;

        // y
        (*object).y = y;

        // return
        mem::size_of::<Point>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut Point = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Point>() as isize);

        // x

        // y

        // return
        (mem::size_of::<Point>(), object.as_mut().unwrap())
    }
}


