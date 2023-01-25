#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

pub struct ListObjectsParameters {
    pub path: String,
    pub pattern: String,
    pub recursive: bool,
}

impl ListObjectsParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ListObjectsParameters, 1);
        pointer = pointer.offset(mem::size_of::<ListObjectsParameters>() as isize);

        mem::size_of::<ListObjectsParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // String path
        let mut len = self.path.len();
        *(pointer as *mut usize) = len;
        core::ptr::copy(self.path.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String pattern
        let mut len = self.pattern.len();
        *(pointer as *mut usize) = len;
        core::ptr::copy(self.pattern.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // Bool recursive

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ListObjectsParameters>() + Self::reconstruct_at(object_pointer as *mut ListObjectsParameters, object_pointer.offset(mem::size_of::<ListObjectsParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ListObjectsParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // String path
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).path), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String pattern
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).pattern), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // Bool recursive

        size
    }
}

#[repr(C, u64)]
pub enum ListObjectsReturnsObjectsEnum {
    TypeDirectory(Directory),
    TypeFile(File),
}

#[repr(C)]
struct ListObjectsReturnsObjectsEnumStruct {
    tag: ListObjectsReturnsObjectsEnumStructTag,
    payload: ListObjectsReturnsObjectsEnumStructPayload,
}

#[repr(u64)]
enum ListObjectsReturnsObjectsEnumStructTag {
    TypeDirectory,
    TypeFile,
}

#[repr(C)]
union ListObjectsReturnsObjectsEnumStructPayload {
    payload_type_directory: ManuallyDrop<Directory>,
    payload_type_file: ManuallyDrop<File>,
}

impl ListObjectsReturnsObjectsEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ListObjectsReturnsObjectsEnum, 1);
        pointer = pointer.offset(mem::size_of::<ListObjectsReturnsObjectsEnum>() as isize);
        mem::size_of::<ListObjectsReturnsObjectsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            ListObjectsReturnsObjectsEnum::TypeDirectory(value) => {
                value.write_references_at(pointer)
            },
            ListObjectsReturnsObjectsEnum::TypeFile(value) => {
                value.write_references_at(pointer)
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ListObjectsReturnsObjectsEnum, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut ListObjectsReturnsObjectsEnumStruct;
        match (*object).tag {
            ListObjectsReturnsObjectsEnumStructTag::TypeDirectory => {
                Directory::reconstruct_at(addr_of_mut!((*object).payload.payload_type_directory) as *mut Directory, references_pointer)
            },
            ListObjectsReturnsObjectsEnumStructTag::TypeFile => {
                File::reconstruct_at(addr_of_mut!((*object).payload.payload_type_file) as *mut File, references_pointer)
            },
        }
    }
}

pub struct ListObjectsReturns {
    pub objects: Vec<ListObjectsReturnsObjectsEnum>,
}

impl ListObjectsReturns {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ListObjectsReturns, 1);
        pointer = pointer.offset(mem::size_of::<ListObjectsReturns>() as isize);

        mem::size_of::<ListObjectsReturns>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // OneOfType objects
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut ListObjectsReturnsObjectsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<ListObjectsReturnsObjectsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<ListObjectsReturnsObjectsEnum>();
        for item in self.objects.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ListObjectsReturns>() + Self::reconstruct_at(object_pointer as *mut ListObjectsReturns, object_pointer.offset(mem::size_of::<ListObjectsReturns>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ListObjectsReturns, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // OneOfType objects
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut ListObjectsReturnsObjectsEnum, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).objects), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<ListObjectsReturnsObjectsEnum>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<ListObjectsReturnsObjectsEnum>() as isize);
        for item in (*object_pointer).objects.iter() {
            let item_size = ListObjectsReturnsObjectsEnum::reconstruct_at(pointer as *mut ListObjectsReturnsObjectsEnum, references_pointer);
            pointer = pointer.offset(mem::size_of::<ListObjectsReturnsObjectsEnum>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



