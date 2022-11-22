use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct PointerPressedParameters {
    position: Point,
    buttons: Vec<PointerButton>,
}
impl PointerPressedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, position_x: i64, position_y: i64, buttons_count: usize) -> (usize, ManuallyDrop<Vec<PointerButton>>) {
        let object: *mut PointerPressedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<PointerPressedParameters>() as isize);

        // position
        (*object).position.x = position_x;
        (*object).position.y = position_y;

        // buttons
        *(pointer as *mut usize) = buttons_count;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let buttons = Vec::<PointerButton>::from_raw_parts(pointer as *mut PointerButton, buttons_count, buttons_count);

        // return
        (mem::size_of::<PointerPressedParameters>() + mem::size_of::<usize>() + buttons_count * mem::size_of::<PointerButton>(), ManuallyDrop::new(buttons))
    }
}


