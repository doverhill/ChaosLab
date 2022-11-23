#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub unsafe fn create_at_address(pointer: *mut u8, alpha: u8, red: u8, green: u8, blue: u8) -> usize {
        let object: *mut Color = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Color>() as isize);

        // alpha
        (*object).alpha = alpha;

        // red
        (*object).red = red;

        // green
        (*object).green = green;

        // blue
        (*object).blue = blue;

        // return
        mem::size_of::<Color>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Color, 1);
        let pointer = pointer.offset(mem::size_of::<Color>() as isize);

        // alpha

        // red

        // green

        // blue

        // return
        mem::size_of::<Color>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Color = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Color>() as isize);

        // alpha

        // red

        // green

        // blue

        // return
        (mem::size_of::<Color>(), object)
    }
}


