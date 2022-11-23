#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Button {
    pub component_id: u64,
    pub parent_component_id: u64,
    pub icon_name: String,
    pub text: String,
}

impl Button {
    pub unsafe fn create_at_address(pointer: *mut u8, component_id: u64, parent_component_id: u64, icon_name: &str, text: &str) -> usize {
        let object: *mut Button = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Button>() as isize);

        // component_id
        (*object).component_id = component_id;

        // parent_component_id
        (*object).parent_component_id = parent_component_id;

        // icon_name
        let _icon_name_length = icon_name.len();
        *(pointer as *mut usize) = _icon_name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(icon_name.as_ptr(), pointer, _icon_name_length);
        let pointer = pointer.offset(_icon_name_length as isize);

        // text
        let _text_length = text.len();
        *(pointer as *mut usize) = _text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(text.as_ptr(), pointer, _text_length);
        let pointer = pointer.offset(_text_length as isize);

        // return
        mem::size_of::<Button>() + mem::size_of::<usize>() + _icon_name_length + mem::size_of::<usize>() + _text_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Button, 1);
        let pointer = pointer.offset(mem::size_of::<Button>() as isize);

        // component_id

        // parent_component_id

        // icon_name
        let _icon_name_length = self.icon_name.len();
        *(pointer as *mut usize) = _icon_name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.icon_name.as_ptr(), pointer, _icon_name_length);
        let pointer = pointer.offset(_icon_name_length as isize);

        // text
        let _text_length = self.text.len();
        *(pointer as *mut usize) = _text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.text.as_ptr(), pointer, _text_length);
        let pointer = pointer.offset(_text_length as isize);

        // return
        mem::size_of::<Button>() + mem::size_of::<usize>() + _icon_name_length + mem::size_of::<usize>() + _text_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Button = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Button>() as isize);

        // component_id

        // parent_component_id

        // icon_name
        let _icon_name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).icon_name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _icon_name_length)).to_owned();
        let pointer = pointer.offset(_icon_name_length as isize);

        // text
        let _text_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _text_length)).to_owned();
        let pointer = pointer.offset(_text_length as isize);

        // return
        (mem::size_of::<Button>() + mem::size_of::<usize>() + _icon_name_length + mem::size_of::<usize>() + _text_length, object)
    }
}


