#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub enum MapFieldValueEnum {
    TypeI64(i64),
    TypeBool(bool),
    TypeString(String),
    TypeNone,
}

impl MapFieldValueEnum {
    pub const OPTION_I64: usize = 1;
    pub const OPTION_BOOL: usize = 2;
    pub const OPTION_STRING: usize = 3;
    pub const OPTION_NONE: usize = 4;

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        let base_pointer = pointer;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let size: usize = mem::size_of::<usize>();

        let size = match self {
            MapFieldValueEnum::TypeI64(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_I64;
                *(pointer as *mut i64) = *value;
                mem::size_of::<usize>()
            },
            MapFieldValueEnum::TypeBool(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_BOOL;
                *(pointer as *mut usize) = if *value { 1 } else { 0 };
                mem::size_of::<usize>()
            },
            MapFieldValueEnum::TypeString(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_STRING;
                let value_len = value.len();
                *(pointer as *mut usize) = value_len;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer as *mut u8, value_len);
                mem::size_of::<usize>() + value_len
            },
            MapFieldValueEnum::TypeNone => {
                *(base_pointer as *mut usize) = Self::OPTION_NONE;
                0
            },
        };

        mem::size_of::<usize>() + size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, Self) {
        let enum_type = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        let (size, object) = match enum_type {
            Self::OPTION_I64 => {
                let value = *(pointer as *mut i64);
                (mem::size_of::<usize>(), Self::TypeI64(value))
            }
            Self::OPTION_BOOL => {
                let value = *(pointer as *mut usize) == 1;
                (mem::size_of::<usize>(), Self::TypeBool(value))
            }
            Self::OPTION_STRING => {
                let value_len = *(pointer as *mut usize);
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let value = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, value_len)).to_owned();
                (mem::size_of::<usize>() + value_len, Self::TypeString(value))
            }
            Self::OPTION_NONE => {
                (0, Self::TypeNone)
            }
            _ => {
                panic!("Unknown enum type");
            }
        };

        (mem::size_of::<usize>() + size, object)
    }
}

pub struct MapField {
    pub name: String,
    pub value: MapFieldValueEnum,
}

impl MapField {
    pub unsafe fn create_at_address(pointer: *mut u8, name: &str, value: MapFieldValueEnum) -> usize {
        let object: *mut MapField = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<MapField>() as isize);

        // name
        let _name_length = name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // value

        // return
        mem::size_of::<MapField>() + mem::size_of::<usize>() + _name_length
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut MapField, 1);
        let pointer = pointer.offset(mem::size_of::<MapField>() as isize);

        // name
        let _name_length = self.name.len();
        *(pointer as *mut usize) = _name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, _name_length);
        let pointer = pointer.offset(_name_length as isize);

        // value

        // return
        mem::size_of::<MapField>() + mem::size_of::<usize>() + _name_length
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut MapField = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<MapField>() as isize);

        // name
        let _name_length = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _name_length)).to_owned();
        let pointer = pointer.offset(_name_length as isize);

        // value

        // return
        (mem::size_of::<MapField>() + mem::size_of::<usize>() + _name_length, object)
    }
}


