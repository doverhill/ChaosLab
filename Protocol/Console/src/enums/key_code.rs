#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(u64)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
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
struct KeyCodeStruct {
    tag: KeyCodeStructTag,
    payload: KeyCodeStructPayload,
}

#[repr(u64)]
enum KeyCodeStructTag {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
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
union KeyCodeStructPayload {
    payload_a: [u8; 0],
    payload_b: [u8; 0],
    payload_c: [u8; 0],
    payload_d: [u8; 0],
    payload_e: [u8; 0],
    payload_f: [u8; 0],
    payload_g: [u8; 0],
    payload_h: [u8; 0],
    payload_i: [u8; 0],
    payload_j: [u8; 0],
    payload_k: [u8; 0],
    payload_l: [u8; 0],
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

impl KeyCode {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut KeyCode, 1);
        pointer = pointer.offset(mem::size_of::<KeyCode>() as isize);
        mem::size_of::<KeyCode>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            KeyCode::A => {
                0
            },
            KeyCode::B => {
                0
            },
            KeyCode::C => {
                0
            },
            KeyCode::D => {
                0
            },
            KeyCode::E => {
                0
            },
            KeyCode::F => {
                0
            },
            KeyCode::G => {
                0
            },
            KeyCode::H => {
                0
            },
            KeyCode::I => {
                0
            },
            KeyCode::J => {
                0
            },
            KeyCode::K => {
                0
            },
            KeyCode::L => {
                0
            },
            KeyCode::Enter => {
                0
            },
            KeyCode::Backspace => {
                0
            },
            KeyCode::LeftArrow => {
                0
            },
            KeyCode::RightArrow => {
                0
            },
            KeyCode::UpArrow => {
                0
            },
            KeyCode::DownArrow => {
                0
            },
            KeyCode::Delete => {
                0
            },
            KeyCode::Home => {
                0
            },
            KeyCode::End => {
                0
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut KeyCode, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut KeyCodeStruct;
        match (*object).tag {
            KeyCodeStructTag::A => {
                0
            },
            KeyCodeStructTag::B => {
                0
            },
            KeyCodeStructTag::C => {
                0
            },
            KeyCodeStructTag::D => {
                0
            },
            KeyCodeStructTag::E => {
                0
            },
            KeyCodeStructTag::F => {
                0
            },
            KeyCodeStructTag::G => {
                0
            },
            KeyCodeStructTag::H => {
                0
            },
            KeyCodeStructTag::I => {
                0
            },
            KeyCodeStructTag::J => {
                0
            },
            KeyCodeStructTag::K => {
                0
            },
            KeyCodeStructTag::L => {
                0
            },
            KeyCodeStructTag::Enter => {
                0
            },
            KeyCodeStructTag::Backspace => {
                0
            },
            KeyCodeStructTag::LeftArrow => {
                0
            },
            KeyCodeStructTag::RightArrow => {
                0
            },
            KeyCodeStructTag::UpArrow => {
                0
            },
            KeyCodeStructTag::DownArrow => {
                0
            },
            KeyCodeStructTag::Delete => {
                0
            },
            KeyCodeStructTag::Home => {
                0
            },
            KeyCodeStructTag::End => {
                0
            },
        }
    }
}



