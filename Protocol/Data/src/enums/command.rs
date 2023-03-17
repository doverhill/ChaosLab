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
pub enum Command {
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
struct CommandStruct {
    tag: CommandStructTag,
    payload: CommandStructPayload,
}

#[repr(u64)]
enum CommandStructTag {
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
union CommandStructPayload {
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

impl Command {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Command, 1);
        pointer = pointer.offset(mem::size_of::<Command>() as isize);
        mem::size_of::<Command>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            Command::Enter => {
                0
            },
            Command::Backspace => {
                0
            },
            Command::LeftArrow => {
                0
            },
            Command::RightArrow => {
                0
            },
            Command::UpArrow => {
                0
            },
            Command::DownArrow => {
                0
            },
            Command::Delete => {
                0
            },
            Command::Home => {
                0
            },
            Command::End => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Command, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut CommandStruct;
        match (*object).tag {
            CommandStructTag::Enter => {
                0
            },
            CommandStructTag::Backspace => {
                0
            },
            CommandStructTag::LeftArrow => {
                0
            },
            CommandStructTag::RightArrow => {
                0
            },
            CommandStructTag::UpArrow => {
                0
            },
            CommandStructTag::DownArrow => {
                0
            },
            CommandStructTag::Delete => {
                0
            },
            CommandStructTag::Home => {
                0
            },
            CommandStructTag::End => {
                0
            },
        }
    }
}



