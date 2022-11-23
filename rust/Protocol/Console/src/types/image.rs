#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Image {
    pub size: Size,
    pub pixels: Vec<Color>,
}

impl Image {
    pub unsafe fn create_at_address(pointer: *mut u8, size_width: u64, size_height: u64, pixels_count: usize) -> (usize, ManuallyDrop<Vec<Color>>) {
        let object: *mut Image = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Image>() as isize);

        // size
        (*object).size.width = size_width;
        (*object).size.height = size_height;

        // pixels
        *(pointer as *mut usize) = pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, pixels_count, pixels_count);
        let pointer = pointer.offset(pixels_count as isize * mem::size_of::<Color>() as isize);

        // return
        (mem::size_of::<Image>() + mem::size_of::<usize>() + pixels_count * mem::size_of::<Color>(), ManuallyDrop::new(pixels))
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Image, 1);
        let pointer = pointer.offset(mem::size_of::<Image>() as isize);

        // size

        // pixels
        let pixels_count = self.pixels.len();
        *(pointer as *mut usize) = pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, pixels_count, pixels_count);
        let pointer = pointer.offset(pixels_count as isize * mem::size_of::<Color>() as isize);

        // return
        mem::size_of::<Image>() + mem::size_of::<usize>() + pixels_count * mem::size_of::<Color>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Image = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Image>() as isize);

        // size

        // pixels
        let pixels_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, pixels_count, pixels_count);
        let pointer = pointer.offset(pixels_count as isize * mem::size_of::<Color>() as isize);
        (*object).pixels = pixels;

        // return
        (mem::size_of::<Image>() + mem::size_of::<usize>() + pixels_count * mem::size_of::<Color>(), object)
    }
}


