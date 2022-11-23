#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct StorageObject {
    pub name: String,
    pub path: String,
}

impl StorageObject {
    pub unsafe fn create_at_address(pointer: *mut u8, name: &str, path: &str) -> usize {
        let object: *mut StorageObject = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<StorageObject>() as isize);

        // name
        let _name_length = name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // path
        let _path_length = path.len();
        *(pointer as *mut usize) = _path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(path.as_ptr(), pointer, _path_length);
        let pointer = pointer.offset(_path_length as isize);

        // return
        mem::size_of::<StorageObject>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _path_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut StorageObject, 1);
        let pointer = pointer.offset(mem::size_of::<StorageObject>() as isize);

        // name
        let _name_length = self.name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // path
        let _path_length = self.path.len();
        *(pointer as *mut usize) = _path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.path.as_ptr(), pointer, _path_length);
        let pointer = pointer.offset(_path_length as isize);

        // return
        mem::size_of::<StorageObject>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _path_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut StorageObject = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<StorageObject>() as isize);

        // name
        let _name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _name_length)).to_owned();
        let pointer = pointer.offset(_name_length as isize);

        // path
        let _path_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _path_length)).to_owned();
        let pointer = pointer.offset(_path_length as isize);

        // return
        (mem::size_of::<StorageObject>() + mem::size_of::<usize>() + _name_length + mem::size_of::<usize>() + _path_length, object)
    }
}


