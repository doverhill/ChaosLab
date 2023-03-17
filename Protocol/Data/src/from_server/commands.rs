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

pub struct CommandsParameters {
    pub commands: Vec<DataCommand>,
}

impl CommandsParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut CommandsParameters, 1);
        pointer = pointer.offset(mem::size_of::<CommandsParameters>() as isize);

        mem::size_of::<CommandsParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // Enum commands
        let len = self.commands.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.commands.as_ptr(), pointer as *mut DataCommand, len);
        pointer = pointer.offset(len as isize * mem::size_of::<DataCommand>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<DataCommand>();

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<CommandsParameters>() + Self::reconstruct_at(object_pointer as *mut CommandsParameters, object_pointer.offset(mem::size_of::<CommandsParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut CommandsParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // Enum commands
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut DataCommand, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).commands), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<DataCommand>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<DataCommand>() as isize);
        pointer = references_pointer;

        size
    }
}



