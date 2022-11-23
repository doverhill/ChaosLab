#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Table {
    pub name: String,
    pub description: String,
    pub columns: Vec<String>,
    pub rows: Vec<*mut Map>,
}

impl Table {
    pub unsafe fn create_at_address(pointer: *mut u8, name: &str, description: &str, columns: Vec<&str>, rows: Vec<Map>) -> usize {
        let object: *mut Table = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Table>() as isize);

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

        // columns
        *(pointer as *mut usize) = columns.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _columns_size: usize = mem::size_of::<usize>();
        for item in columns.iter() {
            let item_size = item.len();
            *(pointer as *mut usize) = item_size;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, item_size);
            let pointer = pointer.offset(item_size as isize);
            _columns_size += mem::size_of::<usize>() + item_size;
        }

        // rows
        *(pointer as *mut usize) = rows.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _rows_size: usize = mem::size_of::<usize>();
        for item in rows.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _rows_size += item_size;
        }

        // return
        mem::size_of::<Table>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _columns_size + _rows_size
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut Table, 1);
        let pointer = pointer.offset(mem::size_of::<Table>() as isize);

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

        // columns
        *(pointer as *mut usize) = self.columns.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _columns_size: usize = mem::size_of::<usize>();
        for item in self.columns.iter() {
            let item_size = item.len();
            *(pointer as *mut usize) = item_size;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, item_size);
            let pointer = pointer.offset(item_size as isize);
            _columns_size += mem::size_of::<usize>() + item_size;
        }

        // rows
        *(pointer as *mut usize) = self.rows.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _rows_size: usize = mem::size_of::<usize>();
        for item in self.rows.iter() {
            let item_size = (item.as_ref().unwrap()).write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _rows_size += item_size;
        }

        // return
        mem::size_of::<Table>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _columns_size + _rows_size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut Table = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<Table>() as isize);

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

        // columns
        let columns_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _columns_size: usize = mem::size_of::<usize>();
        let mut _columns_vec: Vec<String> = Vec::with_capacity(_columns_size);
        for _ in 0..columns_count {
            let item_size = *(pointer as *mut usize);
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            let item = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, item_size)).to_owned();
            _columns_vec.push(item);
            let item_size = mem::size_of::<usize>() + item_size;
            let pointer = pointer.offset(item_size as isize);
            _columns_size += item_size;
        }
        (*object).columns = _columns_vec;

        // rows
        let rows_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _rows_size: usize = mem::size_of::<usize>();
        let mut _rows_vec: Vec<*mut Map> = Vec::with_capacity(_rows_size);
        for _ in 0..rows_count {
            let (item_size, item) = Map::get_from_address(pointer);
            _rows_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _rows_size += item_size;
        }
        (*object).rows = _rows_vec;

        // return
        (mem::size_of::<Table>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _description_length + _columns_size + _rows_size, object)
    }
}


