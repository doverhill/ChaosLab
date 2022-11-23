#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct DrawImagePatchesParameters {
    pub image_patches: Vec<*mut ImagePatch>,
}
impl DrawImagePatchesParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, image_patches: Vec<ImagePatch>) -> usize {
        let object: *mut DrawImagePatchesParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchesParameters>() as isize);

        // image_patches
        *(pointer as *mut usize) = image_patches.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _image_patches_size: usize = mem::size_of::<usize>();
        for item in image_patches.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _image_patches_size += item_size;
        }

        // return
        mem::size_of::<DrawImagePatchesParameters>() + _image_patches_size
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut DrawImagePatchesParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchesParameters>() as isize);

        // image_patches
        let image_patches_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _image_patches_size: usize = mem::size_of::<usize>();
        let mut _image_patches_vec: Vec<*mut ImagePatch> = Vec::with_capacity(_image_patches_size);
        for _ in 0..image_patches_count {
            let (item_size, item) = ImagePatch::get_from_address(pointer);
            _image_patches_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _image_patches_size += item_size;
        }
        (*object).image_patches = _image_patches_vec;

        // return
        (mem::size_of::<DrawImagePatchesParameters>() + _image_patches_size, object)
    }
}


