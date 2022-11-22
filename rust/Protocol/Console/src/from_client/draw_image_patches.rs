use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct DrawImagePatchesParameters {
    image_patches: Vec<ImagePatch>,
}
impl DrawImagePatchesParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, image_patches: Vec<ImagePatch>) -> usize {
        let object: *mut DrawImagePatchesParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DrawImagePatchesParameters>() as isize);

        // image_patches
        *(pointer as *mut usize) = image_patches.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _image_patches_size: usize = 0;
        for item in image_patches.iter() {
            let item_size = item.create_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _image_patches_size += item_size;
        }

        // return
        mem::size_of::<DrawImagePatchesParameters>() + _image_patches_size
    }
}


