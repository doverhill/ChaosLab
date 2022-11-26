#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct DrawImagePatchParameters {
    pub image_patch: ImagePatch,
}

impl DrawImagePatchParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, image_patch_image_size_width: u64, image_patch_image_size_height: u64, image_patch_image_pixels_count: usize, image_patch_position_x: i64, image_patch_position_y: i64) -> (usize, ManuallyDrop<Vec<Color>>) {
        let object: *mut DrawImagePatchParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchParameters>() as isize);

        // image_patch
        (*object).image_patch.image.size.width = image_patch_image_size_width;
        (*object).image_patch.image.size.height = image_patch_image_size_height;
        *(pointer as *mut usize) = image_patch_image_pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_patch_image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_patch_image_pixels_count, image_patch_image_pixels_count);
        let pointer = pointer.offset(image_patch_image_pixels_count as isize * mem::size_of::<Color>() as isize);
        (*object).image_patch.position.x = image_patch_position_x;
        (*object).image_patch.position.y = image_patch_position_y;

        // return
        (mem::size_of::<DrawImagePatchParameters>() + mem::size_of::<usize>() + image_patch_image_pixels_count * mem::size_of::<Color>(), ManuallyDrop::new(image_patch_image_pixels))
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut DrawImagePatchParameters, 1);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchParameters>() as isize);

        // image_patch
        let image_patch_image_pixels_count = self.image_patch.image.pixels.len();
        *(pointer as *mut usize) = image_patch_image_pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_patch_image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_patch_image_pixels_count, image_patch_image_pixels_count);
        let pointer = pointer.offset(image_patch_image_pixels_count as isize * mem::size_of::<Color>() as isize);

        // return
        mem::size_of::<DrawImagePatchParameters>() + mem::size_of::<usize>() + image_patch_image_pixels_count * mem::size_of::<Color>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut DrawImagePatchParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchParameters>() as isize);

        // image_patch
        let image_patch_image_pixels_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_patch_image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_patch_image_pixels_count, image_patch_image_pixels_count);
        let pointer = pointer.offset(image_patch_image_pixels_count as isize * mem::size_of::<Color>() as isize);
        (*object).image_patch.image.pixels = image_patch_image_pixels;

        // return
        (mem::size_of::<DrawImagePatchParameters>() + mem::size_of::<usize>() + image_patch_image_pixels_count * mem::size_of::<Color>(), object)
    }
}


