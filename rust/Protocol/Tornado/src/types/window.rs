#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Window {
    pub component_id: u64,
    pub parent_component_id: u64,
    pub title: String,
}

impl Window {
    pub unsafe fn create_at_address(pointer: *mut u8, component_id: u64, parent_component_id: u64, title: &str) -> usize {
        let object: *mut Window = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Window>() as isize);

        // component_id
        (*object).component_id = component_id;

        // parent_component_id
        (*object).parent_component_id = parent_component_id;

        // title
        let _title_length = title.len();
        *(pointer as *mut usize) = _title_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(title.as_ptr(), pointer, _title_length);
        let pointer = pointer.offset(_title_length as isize);

        // return
        mem::size_of::<Window>() + mem::size_of::<usize>() + _title_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Window, 1);
        let pointer = pointer.offset(mem::size_of::<Window>() as isize);

        // component_id

        // parent_component_id

        // title
        let _title_length = self.title.len();
        *(pointer as *mut usize) = _title_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.title.as_ptr(), pointer, _title_length);
        let pointer = pointer.offset(_title_length as isize);

        // return
        mem::size_of::<Window>() + mem::size_of::<usize>() + _title_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Window = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Window>() as isize);

        // component_id

        // parent_component_id

        // title
        let _title_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).title = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _title_length)).to_owned();
        let pointer = pointer.offset(_title_length as isize);

        // return
        (mem::size_of::<Window>() + mem::size_of::<usize>() + _title_length, object)
    }
}


