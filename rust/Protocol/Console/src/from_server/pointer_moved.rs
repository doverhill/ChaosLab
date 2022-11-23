#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct PointerMovedParameters {
    pub position: Point,
}
impl PointerMovedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, position_x: i64, position_y: i64) -> usize {
        let object: *mut PointerMovedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<PointerMovedParameters>() as isize);

        // position
        (*object).position.x = position_x;
        (*object).position.y = position_y;

        // return
        mem::size_of::<PointerMovedParameters>()
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self) {
        let object: *mut PointerMovedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<PointerMovedParameters>() as isize);

        // position

        // return
        (mem::size_of::<PointerMovedParameters>(), object.as_mut().unwrap())
    }
}


