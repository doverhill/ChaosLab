#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Map {
    pub name: String,
    pub description: String,
    pub fields: Vec<*mut MapField>,
}

impl Map {
    pub unsafe fn create_at_address(pointer: *mut u8, name: &str, description: &str, fields: Vec<MapField>) -> usize {
        let object: *mut Map = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Map>() as isize);

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

        // fields
        *(pointer as *mut usize) = fields.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _fields_size: usize = mem::size_of::<usize>();
        for item in fields.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _fields_size += item_size;
        }

        // return
        mem::size_of::<Map>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _fields_size
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Map, 1);
        let pointer = pointer.offset(mem::size_of::<Map>() as isize);

        // name
        let _name_length = self.name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // description
        let _description_length = self.description.len();
        *(pointer as *mut usize) = _description_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.description.as_ptr(), pointer, _description_length);
        let pointer = pointer.offset(_description_length as isize);

        // fields
        *(pointer as *mut usize) = self.fields.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _fields_size: usize = mem::size_of::<usize>();
        for item in self.fields.iter() {
            let item_size = (item.as_ref().unwrap()).write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _fields_size += item_size;
        }

        // return
        mem::size_of::<Map>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _fields_size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Map = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Map>() as isize);

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

        // fields
        let fields_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _fields_size: usize = mem::size_of::<usize>();
        let mut _fields_vec: Vec<*mut MapField> = Vec::with_capacity(_fields_size);
        for _ in 0..fields_count {
            let (item_size, item) = MapField::get_from_address(pointer);
            _fields_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _fields_size += item_size;
        }
        (*object).fields = _fields_vec;

        // return
        (mem::size_of::<Map>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _fields_size, object)
    }
}


