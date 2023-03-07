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

pub struct ReadObjectParameters {
    pub object: Object,
    pub position: u64,
    pub length: u64,
}

impl ReadObjectParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ReadObjectParameters, 1);
        pointer = pointer.offset(mem::size_of::<ReadObjectParameters>() as isize);

        mem::size_of::<ReadObjectParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType object
        let len = self.object.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // U64 position

        // U64 length

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ReadObjectParameters>() + Self::reconstruct_at(object_pointer as *mut ReadObjectParameters, object_pointer.offset(mem::size_of::<ReadObjectParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ReadObjectParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType object
        let len = Object::reconstruct_at(addr_of_mut!((*object_pointer).object), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // U64 position

        // U64 length

        size
    }
}

pub struct ReadObjectReturns {
    pub data: Vec<u8>,
}

impl ReadObjectReturns {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ReadObjectReturns, 1);
        pointer = pointer.offset(mem::size_of::<ReadObjectReturns>() as isize);

        mem::size_of::<ReadObjectReturns>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U8 data
        let len = self.data.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.data.as_ptr(), pointer as *mut u8, len);
        pointer = pointer.offset(len as isize * mem::size_of::<u8>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<u8>();

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ReadObjectReturns>() + Self::reconstruct_at(object_pointer as *mut ReadObjectReturns, object_pointer.offset(mem::size_of::<ReadObjectReturns>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ReadObjectReturns, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U8 data
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut u8, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).data), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<u8>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<u8>() as isize);
        pointer = references_pointer;

        size
    }
}



