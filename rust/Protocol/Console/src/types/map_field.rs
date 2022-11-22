use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

enum MapFieldValueEnum {
    TypeI64(i64),
    TypeBool(bool),
    TypeString(String),
    TypeNone,
}

impl MapFieldValueEnum {
    pub unsafe fn create_at_address(&self, pointer: *mut u8) -> usize {
        let mut size: usize = mem::size_of::<MapFieldValueEnum>();
        core::ptr::copy(self as *const MapFieldValueEnum, pointer as *mut MapFieldValueEnum, 1);

        match self {
            MapFieldValueEnum::TypeI64(value) => {
                size
            },
            MapFieldValueEnum::TypeBool(value) => {
                size
            },
            MapFieldValueEnum::TypeString(value) => {
                let _value_length = value.len();
                *(pointer as *mut usize) = _value_length;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer, _value_length);
                let pointer = pointer.offset(_value_length as isize);
                size += mem::size_of::<usize>() + _value_length;
                size
            },
            MapFieldValueEnum::TypeNone => {
                size
            },
        }
    }
}
struct MapField {
    name: String,
    value: MapFieldValueEnum,
}


