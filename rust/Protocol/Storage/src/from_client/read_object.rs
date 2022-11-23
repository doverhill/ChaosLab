#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct ReadObjectParameters {
    pub object: StorageObject,
    pub position: u64,
    pub length: u64,
}

impl ReadObjectParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, object_name: &str, object_path: &str, position: u64, length: u64) -> usize {
        let object: *mut ReadObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ReadObjectParameters>() as isize);

        // object
        let _object_name_length = object_name.len();
        *(pointer as *mut usize) = _object_name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(object_name.as_ptr(), pointer, _object_name_length);
        let pointer = pointer.offset(_object_name_length as isize);
        let _object_path_length = object_path.len();
        *(pointer as *mut usize) = _object_path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(object_path.as_ptr(), pointer, _object_path_length);
        let pointer = pointer.offset(_object_path_length as isize);

        // position
        (*object).position = position;

        // length
        (*object).length = length;

        // return
        mem::size_of::<ReadObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut ReadObjectParameters, 1);
        let pointer = pointer.offset(mem::size_of::<ReadObjectParameters>() as isize);

        // object
        let _object_name_length = self.object.name.len();
        *(pointer as *mut usize) = _object_name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.object.name.as_ptr(), pointer, _object_name_length);
        let pointer = pointer.offset(_object_name_length as isize);
        let _object_path_length = self.object.path.len();
        *(pointer as *mut usize) = _object_path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.object.path.as_ptr(), pointer, _object_path_length);
        let pointer = pointer.offset(_object_path_length as isize);

        // position

        // length

        // return
        mem::size_of::<ReadObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut ReadObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ReadObjectParameters>() as isize);

        // object
        let _object_name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).object.name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _object_name_length)).to_owned();
        let pointer = pointer.offset(_object_name_length as isize);
        let _object_path_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).object.path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _object_path_length)).to_owned();
        let pointer = pointer.offset(_object_path_length as isize);

        // position

        // length

        // return
        (mem::size_of::<ReadObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length, object)
    }
}
pub struct ReadObjectReturns {
    pub data: Vec<u8>,
}

impl ReadObjectReturns {
    pub unsafe fn create_at_address(pointer: *mut u8, data_count: usize) -> (usize, ManuallyDrop<Vec<u8>>) {
        let object: *mut ReadObjectReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ReadObjectReturns>() as isize);

        // data
        *(pointer as *mut usize) = data_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let data = Vec::<u8>::from_raw_parts(pointer as *mut u8, data_count, data_count);
        let pointer = pointer.offset(data_count as isize * mem::size_of::<u8>() as isize);

        // return
        (mem::size_of::<ReadObjectReturns>() + mem::size_of::<usize>() + data_count * mem::size_of::<u8>(), ManuallyDrop::new(data))
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut ReadObjectReturns, 1);
        let pointer = pointer.offset(mem::size_of::<ReadObjectReturns>() as isize);

        // data
        let data_count = self.data.len();
        *(pointer as *mut usize) = data_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let data = Vec::<u8>::from_raw_parts(pointer as *mut u8, data_count, data_count);
        let pointer = pointer.offset(data_count as isize * mem::size_of::<u8>() as isize);

        // return
        mem::size_of::<ReadObjectReturns>() + mem::size_of::<usize>() + data_count * mem::size_of::<u8>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut ReadObjectReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ReadObjectReturns>() as isize);

        // data
        let data_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let data = Vec::<u8>::from_raw_parts(pointer as *mut u8, data_count, data_count);
        let pointer = pointer.offset(data_count as isize * mem::size_of::<u8>() as isize);
        (*object).data = data;

        // return
        (mem::size_of::<ReadObjectReturns>() + mem::size_of::<usize>() + data_count * mem::size_of::<u8>(), object)
    }
}


