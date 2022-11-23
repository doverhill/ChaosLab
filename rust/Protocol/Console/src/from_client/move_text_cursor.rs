#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct MoveTextCursorParameters {
    pub position: Point,
}
impl MoveTextCursorParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, position_x: i64, position_y: i64) -> usize {
        let object: *mut MoveTextCursorParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<MoveTextCursorParameters>() as isize);

        // position
        (*object).position.x = position_x;
        (*object).position.y = position_y;

        // return
        mem::size_of::<MoveTextCursorParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut MoveTextCursorParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<MoveTextCursorParameters>() as isize);

        // position

        // return
        (mem::size_of::<MoveTextCursorParameters>(), object.as_mut().unwrap())
    }
}


