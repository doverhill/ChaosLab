#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct WriteTextParameters {
    pub text: String,
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

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut WriteTextParameters, 1);
        let pointer = pointer.offset(mem::size_of::<WriteTextParameters>() as isize);

        // text
        let _text_length = self.text.len();
        *(pointer as *mut usize) = _text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.text.as_ptr(), pointer, _text_length);
        let pointer = pointer.offset(_text_length as isize);

        // return
        mem::size_of::<WriteTextParameters>() + mem::size_of::<usize>() + _text_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut WriteTextParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteTextParameters>() as isize);

        // text
        let _text_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _text_length)).to_owned();
        let pointer = pointer.offset(_text_length as isize);

        // return
        (mem::size_of::<WriteTextParameters>() + mem::size_of::<usize>() + _text_length, object)
    }
}


