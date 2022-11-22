use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct GetCapabilitiesReturns {
    is_framebuffer: bool,
    framebuffer_size: Size,
    text_size: Size,
}
impl GetCapabilitiesReturns {
    pub unsafe fn create_at_address(pointer: *mut u8, is_framebuffer: bool, framebuffer_size_width: u64, framebuffer_size_height: u64, text_size_width: u64, text_size_height: u64) -> usize {
        let object: *mut GetCapabilitiesReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<GetCapabilitiesReturns>() as isize);

        // is_framebuffer
        (*object).is.framebuffer = is_framebuffer;

        // framebuffer_size
        (*object).framebuffer.size.width = framebuffer_size_width;
        (*object).framebuffer.size.height = framebuffer_size_height;

        // text_size
        (*object).text.size.width = text_size_width;
        (*object).text.size.height = text_size_height;

        // return
        mem::size_of::<GetCapabilitiesReturns>()
    }
}


