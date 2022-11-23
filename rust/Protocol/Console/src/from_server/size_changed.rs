#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct SizeChangedParameters {
    pub framebuffer_size: Size,
    pub text_size: Size,
}
impl SizeChangedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, framebuffer_size_width: u64, framebuffer_size_height: u64, text_size_width: u64, text_size_height: u64) -> usize {
        let object: *mut SizeChangedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SizeChangedParameters>() as isize);

        // framebuffer_size
        (*object).framebuffer_size.width = framebuffer_size_width;
        (*object).framebuffer_size.height = framebuffer_size_height;

        // text_size
        (*object).text_size.width = text_size_width;
        (*object).text_size.height = text_size_height;

        // return
        mem::size_of::<SizeChangedParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut SizeChangedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SizeChangedParameters>() as isize);

        // framebuffer_size

        // text_size

        // return
        (mem::size_of::<SizeChangedParameters>(), object.as_mut().unwrap())
    }
}


