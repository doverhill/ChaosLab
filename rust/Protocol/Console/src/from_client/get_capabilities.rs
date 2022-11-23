#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct GetCapabilitiesReturns {
    pub is_framebuffer: bool,
    pub framebuffer_size: Size,
    pub text_size: Size,
}
impl GetCapabilitiesReturns {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, is_framebuffer: bool, framebuffer_size_width: u64, framebuffer_size_height: u64, text_size_width: u64, text_size_height: u64) -> usize {
        let object: *mut GetCapabilitiesReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<GetCapabilitiesReturns>() as isize);

        // is_framebuffer
        (*object).is_framebuffer = is_framebuffer;

        // framebuffer_size
        (*object).framebuffer_size.width = framebuffer_size_width;
        (*object).framebuffer_size.height = framebuffer_size_height;

        // text_size
        (*object).text_size.width = text_size_width;
        (*object).text_size.height = text_size_height;

        // return
        mem::size_of::<GetCapabilitiesReturns>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut GetCapabilitiesReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<GetCapabilitiesReturns>() as isize);

        // is_framebuffer

        // framebuffer_size

        // text_size

        // return
        (mem::size_of::<GetCapabilitiesReturns>(), object)
    }
}


