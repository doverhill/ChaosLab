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

#[derive(Copy, Clone)]
pub struct ReadParameters {
    pub block_index: u64,
    pub block_count: u64,
}

impl ReadParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ReadParameters, 1);
        pointer = pointer.offset(mem::size_of::<ReadParameters>() as isize);

        mem::size_of::<ReadParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 block_index

        // U64 block_count

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<ReadParameters>() + Self::reconstruct_at(object_pointer as *mut ReadParameters, object_pointer.offset(mem::size_of::<ReadParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ReadParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 block_index

        // U64 block_count

        size
    }
}

pub struct ReadReturns {
    pub data: Vec<u8>,
}

impl ReadReturns {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut ReadReturns, 1);
        pointer = pointer.offset(mem::size_of::<ReadReturns>() as isize);

        mem::size_of::<ReadReturns>() + self.write_references_at(pointer)
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
        mem::size_of::<ReadReturns>() + Self::reconstruct_at(object_pointer as *mut ReadReturns, object_pointer.offset(mem::size_of::<ReadReturns>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut ReadReturns, references_pointer: *mut u8) -> usize {
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



