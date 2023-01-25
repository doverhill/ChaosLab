#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

pub struct StorageObject {
    pub name: String,
    pub path: String,
}

impl StorageObject {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut StorageObject, 1);
        pointer = pointer.offset(mem::size_of::<StorageObject>() as isize);

        mem::size_of::<StorageObject>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // String name
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String path
        let mut len = self.path.len();
        *(pointer as *mut usize) = len;
        core::ptr::copy(self.path.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<StorageObject>() + Self::reconstruct_at(object_pointer as *mut StorageObject, object_pointer.offset(mem::size_of::<StorageObject>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut StorageObject, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // String name
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).name), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String path
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).path), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        size
    }
}



