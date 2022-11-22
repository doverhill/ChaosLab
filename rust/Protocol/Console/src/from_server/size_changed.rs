struct SizeChangedParameters {
    framebuffer_size: Size,
    text_size: Size,
}


impl SizeChangedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, framebuffer_size_width: u64, framebuffer_size_height: u64, text_size_width: u64, text_size_height: u64) -> usize {
        let object: *mut SizeChangedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SizeChangedParameters>() as isize);

        // framebuffer_size
        (*object).framebuffer.size.width = framebuffer_size_width;
        (*object).framebuffer.size.height = framebuffer_size_height;

        // text_size
        (*object).text.size.width = text_size_width;
        (*object).text.size.height = text_size_height;

        // return
        mem::size_of::<SizeChangedParameters>()
    }
}



