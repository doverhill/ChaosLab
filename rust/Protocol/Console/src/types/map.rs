#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Map {
    pub name: String,
    pub description: String,
    pub fields: Vec<MapField>,
}
impl Map {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, fields: Vec<MapField>) -> usize {
        let object: *mut Map = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Map>() as isize);

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
        mem::size_of::<Map>() + _fields_size
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut Map = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Map>() as isize);

        // fields
        let fields_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _fields_size: usize = mem::size_of::<usize>();
        let mut _fields_vec: Vec<MapField> = Vec::with_capacity(_fields_size);
        for _ in 0..fields_count {
            let (item_size, item) = MapField::get_from_address(pointer);
            _fields_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _fields_size += item_size;
        }
        (*object).fields = _fields_vec;

        // return
        (mem::size_of::<Map>() + _fields_size, object.as_mut().unwrap())
    }
}


