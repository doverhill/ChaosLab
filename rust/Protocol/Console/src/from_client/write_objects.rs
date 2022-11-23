#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub enum WriteObjectsParametersObjectsEnum {
    TypeTable(*mut Table),
    TypeMap(*mut Map),
}

impl WriteObjectsParametersObjectsEnum {
    pub const OPTION_TABLE: usize = 1;
    pub const OPTION_MAP: usize = 2;

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        let base_pointer = pointer;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let size: usize = mem::size_of::<usize>();

        let size = match self {
            WriteObjectsParametersObjectsEnum::TypeTable(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_TABLE;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            WriteObjectsParametersObjectsEnum::TypeMap(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_MAP;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
        };

        mem::size_of::<usize>() + size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, Self) {
        let enum_type = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        let (size, object) = match enum_type {
            Self::OPTION_TABLE => {
                let (size, value) = Table::get_from_address(pointer);
                (size, Self::TypeTable(value))
            }
            Self::OPTION_MAP => {
                let (size, value) = Map::get_from_address(pointer);
                (size, Self::TypeMap(value))
            }
            _ => {
                panic!("Unknown enum type");
            }
        };

        (mem::size_of::<usize>() + size, object)
    }
}

pub struct WriteObjectsParameters {
    pub objects: Vec<WriteObjectsParametersObjectsEnum>,
}

impl WriteObjectsParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, objects: Vec<WriteObjectsParametersObjectsEnum>) -> usize {
        let object: *mut WriteObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

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
        mem::size_of::<WriteObjectsParameters>() + _objects_size
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut WriteObjectsParameters, 1);
        let pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

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
        mem::size_of::<WriteObjectsParameters>() + _objects_size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut WriteObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

        // objects
        let objects_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = mem::size_of::<usize>();
        let mut _objects_vec: Vec<WriteObjectsParametersObjectsEnum> = Vec::with_capacity(_objects_size);
        for _ in 0..objects_count {
            let (item_size, item) = WriteObjectsParametersObjectsEnum::get_from_address(pointer);
            _objects_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }
        (*object).objects = _objects_vec;

        // return
        (mem::size_of::<WriteObjectsParameters>() + _objects_size, object)
    }
}


