struct DebugDrawImageParameters {
    image: Image,
}


impl DebugDrawImageParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, image_size_width: u64, image_size_height: u64, image_pixels_count: usize) -> (usize, ManuallyDrop<Vec<Color>>) {
        let object: *mut DebugDrawImageParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<DebugDrawImageParameters>() as isize);

        // image
        (*object).image.size.width = image_size_width;
        (*object).image.size.height = image_size_height;
        *(pointer as *mut usize) = image_pixels_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let image_pixels = Vec::<Color>::from_raw_parts(pointer as *mut Color, image_pixels_count, image_pixels_count);

        // return
        (mem::size_of::<DebugDrawImageParameters>() + mem::size_of::<usize>() + image_pixels_count * mem::size_of::<Color>(), ManuallyDrop::new(image_pixels))
    }
}



