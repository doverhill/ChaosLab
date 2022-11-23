#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct DebugDrawImageParameters {
    pub image: Image,
}
impl DebugDrawImageParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, image_size_width: u64, image_size_height: u64, image_pixels_count: usize) -> (usize, ManuallyDrop<Vec<Color>>) {
        let object: *mut DebugDrawImageParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DebugDrawImageParameters>() as isize);

        // image
        (*object).image.size.width = image_size_width;
        (*object).image.size.height = image_size_height;
        *(pointer as *mut usize) = image_pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_pixels_count, image_pixels_count);
        let pointer = pointer.offset(image_pixels_count as isize * mem::size_of::<Color>() as isize);

        // return
        (mem::size_of::<DebugDrawImageParameters>() + mem::size_of::<usize>() + image_pixels_count * mem::size_of::<Color>(), ManuallyDrop::new(image_pixels))
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut DebugDrawImageParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DebugDrawImageParameters>() as isize);

        // image
        let image_pixels_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_pixels_count, image_pixels_count);
        let pointer = pointer.offset(image_pixels_count as isize * mem::size_of::<Color>() as isize);
        (*object).image.pixels = image_pixels;

        // return
        (mem::size_of::<DebugDrawImageParameters>() + mem::size_of::<usize>() + image_pixels_count * mem::size_of::<Color>(), object.as_mut().unwrap())
    }
}


