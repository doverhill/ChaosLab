use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

enum WriteObjectsParametersObjectsEnum {
    TypeTable(Table),
    TypeMap(Map),
}

impl WriteObjectsParametersObjectsEnum {
    pub unsafe fn create_at_address(&self, pointer: *mut u8) -> usize {
        let mut size: usize = mem::size_of::<WriteObjectsParametersObjectsEnum>();
        core::ptr::copy(self as *const WriteObjectsParametersObjectsEnum, pointer as *mut WriteObjectsParametersObjectsEnum, 1);

        match self {
            WriteObjectsParametersObjectsEnum::TypeTable(value) => {
                *(pointer as *mut usize) = value.columns.len();
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let mut _columns_size: usize = 0;
                for item in columns.iter() {
                    let item_size = item.create_at_address(pointer);
                    let pointer = pointer.offset(item_size as isize);
                    _columns_size += item_size;
                }
                *(pointer as *mut usize) = value.rows.len();
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let mut _rows_size: usize = 0;
                for item in rows.iter() {
                    let item_size = item.create_at_address(pointer);
                    let pointer = pointer.offset(item_size as isize);
                    _rows_size += item_size;
                }
                size += _columns_size + _rows_size;
                size
            },
            WriteObjectsParametersObjectsEnum::TypeMap(value) => {
                *(pointer as *mut usize) = value.fields.len();
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let mut _fields_size: usize = 0;
                for item in fields.iter() {
                    let item_size = item.create_at_address(pointer);
                    let pointer = pointer.offset(item_size as isize);
                    _fields_size += item_size;
                }
                size += _fields_size;
                size
            },
        }
    }
}
struct WriteObjectsParameters {
    objects: Vec<WriteObjectsParametersObjectsEnum>,
}
impl WriteObjectsParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, objects: Vec<WriteObjectsParametersObjectsEnum>) -> usize {
        let object: *mut WriteObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

        // objects
        *(pointer as *mut usize) = objects.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = 0;
        for item in objects.iter() {
            let item_size = item.create_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }

        // return
        mem::size_of::<WriteObjectsParameters>() + _objects_size
    }
}


