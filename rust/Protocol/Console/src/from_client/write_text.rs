use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct WriteTextParameters {
    text: String,
}
impl WriteTextParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, text: &str) -> usize {
        let object: *mut WriteTextParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteTextParameters>() as isize);

        // text
        let _text_length = text.len();
        *(pointer as *mut usize) = _text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(text.as_ptr(), pointer, _text_length);
        let pointer = pointer.offset(_text_length as isize);

        // return
        mem::size_of::<WriteTextParameters>() + mem::size_of::<usize>() + _text_length
    }
}


