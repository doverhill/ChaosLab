#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct PointerReleasedParameters {
    pub position: Point,
    pub buttons: Vec<PointerButton>,
}
impl PointerReleasedParameters {
    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        0
    }
    pub unsafe fn create_at_address(pointer: *mut u8, position_x: i64, position_y: i64, buttons_count: usize) -> (usize, ManuallyDrop<Vec<PointerButton>>) {
        let object: *mut PointerReleasedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<PointerReleasedParameters>() as isize);

        // position
        (*object).position.x = position_x;
        (*object).position.y = position_y;

        // buttons
        *(pointer as *mut usize) = buttons_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let buttons = Vec::<PointerButton>::from_raw_parts(pointer as *mut PointerButton, buttons_count, buttons_count);
        let pointer = pointer.offset(buttons_count as isize * mem::size_of::<PointerButton>() as isize);

        // return
        (mem::size_of::<PointerReleasedParameters>() + mem::size_of::<usize>() + buttons_count * mem::size_of::<PointerButton>(), ManuallyDrop::new(buttons))
    }
    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut PointerReleasedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<PointerReleasedParameters>() as isize);

        // position

        // buttons
        let buttons_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let buttons = Vec::<PointerButton>::from_raw_parts(pointer as *mut PointerButton, buttons_count, buttons_count);
        let pointer = pointer.offset(buttons_count as isize * mem::size_of::<PointerButton>() as isize);
        (*object).buttons = buttons;

        // return
        (mem::size_of::<PointerReleasedParameters>() + mem::size_of::<usize>() + buttons_count * mem::size_of::<PointerButton>(), object)
    }
}


