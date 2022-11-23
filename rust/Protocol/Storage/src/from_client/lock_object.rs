#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct LockObjectParameters {
    pub object: StorageObject,
}

impl LockObjectParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, object_name: &str, object_path: &str) -> usize {
        let object: *mut LockObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<LockObjectParameters>() as isize);

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

        // return
        mem::size_of::<LockObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut LockObjectParameters, 1);
        let pointer = pointer.offset(mem::size_of::<LockObjectParameters>() as isize);

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

        // return
        mem::size_of::<LockObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut LockObjectParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<LockObjectParameters>() as isize);

        // object
        let _object_name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).object.name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _object_name_length)).to_owned();
        let pointer = pointer.offset(_object_name_length as isize);
        let _object_path_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).object.path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _object_path_length)).to_owned();
        let pointer = pointer.offset(_object_path_length as isize);

        // return
        (mem::size_of::<LockObjectParameters>() + mem::size_of::<usize>() + _object_name_length + mem::size_of::<usize>() + _object_path_length, object)
    }
}
pub struct LockObjectReturns {
    pub lock_id: u64,
}

impl LockObjectReturns {
    pub unsafe fn create_at_address(pointer: *mut u8, lock_id: u64) -> usize {
        let object: *mut LockObjectReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<LockObjectReturns>() as isize);

        // lock_id
        (*object).lock_id = lock_id;

        // return
        mem::size_of::<LockObjectReturns>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut LockObjectReturns, 1);
        let pointer = pointer.offset(mem::size_of::<LockObjectReturns>() as isize);

        // lock_id

        // return
        mem::size_of::<LockObjectReturns>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut LockObjectReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<LockObjectReturns>() as isize);

        // lock_id

        // return
        (mem::size_of::<LockObjectReturns>(), object)
    }
}


