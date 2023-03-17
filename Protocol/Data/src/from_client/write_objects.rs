#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use alloc::vec::Vec;
use alloc::string::String;
use crate::types::*;
use crate::enums::*;

#[repr(C, u64)]
pub enum WriteObjectsParametersObjectsEnum {
    TypeTable(Table),
    TypeMap(Map),
}

#[repr(C)]
struct WriteObjectsParametersObjectsEnumStruct {
    tag: WriteObjectsParametersObjectsEnumStructTag,
    payload: WriteObjectsParametersObjectsEnumStructPayload,
}

#[repr(u64)]
enum WriteObjectsParametersObjectsEnumStructTag {
    TypeTable,
    TypeMap,
}

#[repr(C)]
union WriteObjectsParametersObjectsEnumStructPayload {
    payload_type_table: ManuallyDrop<Table>,
    payload_type_map: ManuallyDrop<Map>,
}

impl WriteObjectsParametersObjectsEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut WriteObjectsParametersObjectsEnum, 1);
        pointer = pointer.offset(mem::size_of::<WriteObjectsParametersObjectsEnum>() as isize);
        mem::size_of::<WriteObjectsParametersObjectsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            WriteObjectsParametersObjectsEnum::TypeTable(value) => {
                value.write_references_at(pointer)
            },
            WriteObjectsParametersObjectsEnum::TypeMap(value) => {
                value.write_references_at(pointer)
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut WriteObjectsParametersObjectsEnum, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut WriteObjectsParametersObjectsEnumStruct;
        match (*object).tag {
            WriteObjectsParametersObjectsEnumStructTag::TypeTable => {
                Table::reconstruct_at(addr_of_mut!((*object).payload.payload_type_table) as *mut Table, references_pointer)
            },
            WriteObjectsParametersObjectsEnumStructTag::TypeMap => {
                Map::reconstruct_at(addr_of_mut!((*object).payload.payload_type_map) as *mut Map, references_pointer)
            },
        }
    }
}

pub struct WriteObjectsParameters {
    pub objects: Vec<WriteObjectsParametersObjectsEnum>,
}

impl WriteObjectsParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut WriteObjectsParameters, 1);
        pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

        mem::size_of::<WriteObjectsParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // OneOfType objects
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut WriteObjectsParametersObjectsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<WriteObjectsParametersObjectsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<WriteObjectsParametersObjectsEnum>();
        for item in self.objects.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<WriteObjectsParameters>() + Self::reconstruct_at(object_pointer as *mut WriteObjectsParameters, object_pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut WriteObjectsParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // OneOfType objects
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut WriteObjectsParametersObjectsEnum, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).objects), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<WriteObjectsParametersObjectsEnum>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<WriteObjectsParametersObjectsEnum>() as isize);
        for item in (*object_pointer).objects.iter() {
            let item_size = WriteObjectsParametersObjectsEnum::reconstruct_at(pointer as *mut WriteObjectsParametersObjectsEnum, references_pointer);
            pointer = pointer.offset(mem::size_of::<WriteObjectsParametersObjectsEnum>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



