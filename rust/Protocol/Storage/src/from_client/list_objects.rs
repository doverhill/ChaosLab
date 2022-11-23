#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
pub struct ListObjectsParameters {
    pub path: String,
    pub pattern: String,
    pub recursive: bool,
}

impl ListObjectsParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, path: &str, pattern: &str, recursive: bool) -> usize {
        let object: *mut ListObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ListObjectsParameters>() as isize);

        // path
        let _path_length = path.len();
        *(pointer as *mut usize) = _path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(path.as_ptr(), pointer, _path_length);
        let pointer = pointer.offset(_path_length as isize);

        // pattern
        let _pattern_length = pattern.len();
        *(pointer as *mut usize) = _pattern_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(pattern.as_ptr(), pointer, _pattern_length);
        let pointer = pointer.offset(_pattern_length as isize);

        // recursive
        (*object).recursive = recursive;

        // return
        mem::size_of::<ListObjectsParameters>() + mem::size_of::<usize>() + _path_length + mem::size_of::<usize>() + _pattern_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut ListObjectsParameters, 1);
        let pointer = pointer.offset(mem::size_of::<ListObjectsParameters>() as isize);

        // path
        let _path_length = self.path.len();
        *(pointer as *mut usize) = _path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.path.as_ptr(), pointer, _path_length);
        let pointer = pointer.offset(_path_length as isize);

        // pattern
        let _pattern_length = self.pattern.len();
        *(pointer as *mut usize) = _pattern_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.pattern.as_ptr(), pointer, _pattern_length);
        let pointer = pointer.offset(_pattern_length as isize);

        // recursive

        // return
        mem::size_of::<ListObjectsParameters>() + mem::size_of::<usize>() + _path_length + mem::size_of::<usize>() + _pattern_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut ListObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ListObjectsParameters>() as isize);

        // path
        let _path_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _path_length)).to_owned();
        let pointer = pointer.offset(_path_length as isize);

        // pattern
        let _pattern_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).pattern = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _pattern_length)).to_owned();
        let pointer = pointer.offset(_pattern_length as isize);

        // recursive

        // return
        (mem::size_of::<ListObjectsParameters>() + mem::size_of::<usize>() + _path_length + mem::size_of::<usize>() + _pattern_length, object)
    }
}
pub enum ListObjectsReturnsObjectsEnum {
    TypeDirectory(*mut Directory),
    TypeFile(*mut File),
}

impl ListObjectsReturnsObjectsEnum {
    pub const OPTION_DIRECTORY: usize = 1;
    pub const OPTION_FILE: usize = 2;

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        let base_pointer = pointer;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let size: usize = mem::size_of::<usize>();

        let size = match self {
            ListObjectsReturnsObjectsEnum::TypeDirectory(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_DIRECTORY;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            ListObjectsReturnsObjectsEnum::TypeFile(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_FILE;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
        };

        mem::size_of::<usize>() + size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, Self) {
        let enum_type = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        let (size, object) = match enum_type {
            Self::OPTION_DIRECTORY => {
                let (size, value) = Directory::get_from_address(pointer);
                (size, Self::TypeDirectory(value))
            }
            Self::OPTION_FILE => {
                let (size, value) = File::get_from_address(pointer);
                (size, Self::TypeFile(value))
            }
            _ => {
                panic!("Unknown enum type");
            }
        };

        (mem::size_of::<usize>() + size, object)
    }
}

pub struct ListObjectsReturns {
    pub objects: Vec<ListObjectsReturnsObjectsEnum>,
}

impl ListObjectsReturns {
    pub unsafe fn create_at_address(pointer: *mut u8, objects: Vec<ListObjectsReturnsObjectsEnum>) -> usize {
        let object: *mut ListObjectsReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ListObjectsReturns>() as isize);

        // objects
        *(pointer as *mut usize) = objects.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = mem::size_of::<usize>();
        for item in objects.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }

        // return
        mem::size_of::<ListObjectsReturns>() + _objects_size
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut ListObjectsReturns, 1);
        let pointer = pointer.offset(mem::size_of::<ListObjectsReturns>() as isize);

        // objects
        *(pointer as *mut usize) = self.objects.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = mem::size_of::<usize>();
        for item in self.objects.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }

        // return
        mem::size_of::<ListObjectsReturns>() + _objects_size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut ListObjectsReturns = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ListObjectsReturns>() as isize);

        // objects
        let objects_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = mem::size_of::<usize>();
        let mut _objects_vec: Vec<ListObjectsReturnsObjectsEnum> = Vec::with_capacity(_objects_size);
        for _ in 0..objects_count {
            let (item_size, item) = ListObjectsReturnsObjectsEnum::get_from_address(pointer);
            _objects_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }
        (*object).objects = _objects_vec;

        // return
        (mem::size_of::<ListObjectsReturns>() + _objects_size, object)
    }
}


