#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct DebugCallParameters {
    pub image: Image,
    pub x: Vec<bool>,
    pub y: Vec<u64>,
    pub z: Vec<String>,
}
impl DebugCallParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, image_size_width: u64, image_size_height: u64, image_pixels_count: usize, x_count: usize, y_count: usize, z: Vec<&str>) -> (usize, ManuallyDrop<Vec<Color>>, ManuallyDrop<Vec<bool>>, ManuallyDrop<Vec<u64>>) {
        let object: *mut DebugCallParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DebugCallParameters>() as isize);

        // image
        (*object).image.size.width = image_size_width;
        (*object).image.size.height = image_size_height;
        *(pointer as *mut usize) = image_pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_pixels_count, image_pixels_count);
        let pointer = pointer.offset(image_pixels_count as isize * mem::size_of::<Color>() as isize);

        // x
        *(pointer as *mut usize) = x_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let x = Vec::<bool>::from_raw_parts(pointer as *mut bool, x_count, x_count);
        let pointer = pointer.offset(x_count as isize * mem::size_of::<bool>() as isize);

        // y
        *(pointer as *mut usize) = y_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let y = Vec::<u64>::from_raw_parts(pointer as *mut u64, y_count, y_count);
        let pointer = pointer.offset(y_count as isize * mem::size_of::<u64>() as isize);

        // z
        *(pointer as *mut usize) = z.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _z_size: usize = mem::size_of::<usize>();
        for item in z.iter() {
            let item_size = item.len();
            *(pointer as *mut usize) = item_size;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, item_size);
            let pointer = pointer.offset(item_size as isize);
            _z_size += mem::size_of::<usize>() + item_size;
        }

        // return
        (mem::size_of::<DebugCallParameters>() + mem::size_of::<usize>() + image_pixels_count * mem::size_of::<Color>() + mem::size_of::<usize>() + x_count * mem::size_of::<bool>() + mem::size_of::<usize>() + y_count * mem::size_of::<u64>() + _z_size, ManuallyDrop::new(image_pixels), ManuallyDrop::new(x), ManuallyDrop::new(y))
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut DebugCallParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DebugCallParameters>() as isize);

        // image
        let image_pixels_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_pixels_count, image_pixels_count);
        let pointer = pointer.offset(image_pixels_count as isize * mem::size_of::<Color>() as isize);
        (*object).image.pixels = image_pixels;

        // x
        let x_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let x = Vec::<bool>::from_raw_parts(pointer as *mut bool, x_count, x_count);
        let pointer = pointer.offset(x_count as isize * mem::size_of::<bool>() as isize);
        (*object).x = x;

        // y
        let y_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let y = Vec::<u64>::from_raw_parts(pointer as *mut u64, y_count, y_count);
        let pointer = pointer.offset(y_count as isize * mem::size_of::<u64>() as isize);
        (*object).y = y;

        // z
        let z_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _z_size: usize = mem::size_of::<usize>();
        let mut _z_vec: Vec<String> = Vec::with_capacity(_z_size);
        for _ in 0..z_count {
            let item_size = *(pointer as *mut usize);
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            let item = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, item_size)).to_owned();
            _z_vec.push(item);
            let item_size = mem::size_of::<usize>() + item_size;
            let pointer = pointer.offset(item_size as isize);
            _z_size += item_size;
        }
        (*object).z = _z_vec;

        // return
        (mem::size_of::<DebugCallParameters>() + mem::size_of::<usize>() + image_pixels_count * mem::size_of::<Color>() + mem::size_of::<usize>() + x_count * mem::size_of::<bool>() + mem::size_of::<usize>() + y_count * mem::size_of::<u64>() + _z_size, object)
    }
}


