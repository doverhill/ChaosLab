#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Object {
    pub name: String,
    pub description: String,
}
impl Object {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, name: &str, description: &str) -> usize {
        let object: *mut Object = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Object>() as isize);

        // name
        let _name_length = name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // description
        let _description_length = description.len();
        *(pointer as *mut usize) = _description_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(description.as_ptr(), pointer, _description_length);
        let pointer = pointer.offset(_description_length as isize);

        // return
        mem::size_of::<Object>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Object = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Object>() as isize);

        // name
        let _name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _name_length)).to_owned();
        let pointer = pointer.offset(_name_length as isize);

        // description
        let _description_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).description = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _description_length)).to_owned();
        let pointer = pointer.offset(_description_length as isize);

        // return
        (mem::size_of::<Object>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length, object)
    }
}


