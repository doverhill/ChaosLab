#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct SetTextColorParameters {
    pub foreground: Color,
    pub background: Color,
}
impl SetTextColorParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, foreground_alpha: u8, foreground_red: u8, foreground_green: u8, foreground_blue: u8, background_alpha: u8, background_red: u8, background_green: u8, background_blue: u8) -> usize {
        let object: *mut SetTextColorParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SetTextColorParameters>() as isize);

        // foreground
        (*object).foreground.alpha = foreground_alpha;
        (*object).foreground.red = foreground_red;
        (*object).foreground.green = foreground_green;
        (*object).foreground.blue = foreground_blue;

        // background
        (*object).background.alpha = background_alpha;
        (*object).background.red = background_red;
        (*object).background.green = background_green;
        (*object).background.blue = background_blue;

        // return
        mem::size_of::<SetTextColorParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut SetTextColorParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SetTextColorParameters>() as isize);

        // foreground

        // background

        // return
        (mem::size_of::<SetTextColorParameters>(), object)
    }
}


