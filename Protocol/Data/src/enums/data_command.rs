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

#[repr(u64)]
pub enum DataCommand {
    Enter,
    Backspace,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Delete,
    Home,
    End,
}

#[repr(C)]
struct DataCommandStruct {
    tag: DataCommandStructTag,
    payload: DataCommandStructPayload,
}

#[repr(u64)]
enum DataCommandStructTag {
    Enter,
    Backspace,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Delete,
    Home,
    End,
}

#[repr(C)]
union DataCommandStructPayload {
    payload_enter: [u8; 0],
    payload_backspace: [u8; 0],
    payload_left_arrow: [u8; 0],
    payload_right_arrow: [u8; 0],
    payload_up_arrow: [u8; 0],
    payload_down_arrow: [u8; 0],
    payload_delete: [u8; 0],
    payload_home: [u8; 0],
    payload_end: [u8; 0],
}

impl DataCommand {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut DataCommand, 1);
        pointer = pointer.offset(mem::size_of::<DataCommand>() as isize);
        mem::size_of::<DataCommand>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            DataCommand::Enter => {
                0
            },
            DataCommand::Backspace => {
                0
            },
            DataCommand::LeftArrow => {
                0
            },
            DataCommand::RightArrow => {
                0
            },
            DataCommand::UpArrow => {
                0
            },
            DataCommand::DownArrow => {
                0
            },
            DataCommand::Delete => {
                0
            },
            DataCommand::Home => {
                0
            },
            DataCommand::End => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut DataCommand, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut DataCommandStruct;
        match (*object).tag {
            DataCommandStructTag::Enter => {
                0
            },
            DataCommandStructTag::Backspace => {
                0
            },
            DataCommandStructTag::LeftArrow => {
                0
            },
            DataCommandStructTag::RightArrow => {
                0
            },
            DataCommandStructTag::UpArrow => {
                0
            },
            DataCommandStructTag::DownArrow => {
                0
            },
            DataCommandStructTag::Delete => {
                0
            },
            DataCommandStructTag::Home => {
                0
            },
            DataCommandStructTag::End => {
                0
            },
        }
    }
}



