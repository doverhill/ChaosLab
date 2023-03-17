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

pub struct CommandParameters {
    pub command: Vec<Command>,
}

impl CommandParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut CommandParameters, 1);
        pointer = pointer.offset(mem::size_of::<CommandParameters>() as isize);

        mem::size_of::<CommandParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // Enum command
        let len = self.command.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.command.as_ptr(), pointer as *mut Command, len);
        pointer = pointer.offset(len as isize * mem::size_of::<Command>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<Command>();

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<CommandParameters>() + Self::reconstruct_at(object_pointer as *mut CommandParameters, object_pointer.offset(mem::size_of::<CommandParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut CommandParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // Enum command
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut Command, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).command), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<Command>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<Command>() as isize);
        pointer = references_pointer;

        size
    }
}



