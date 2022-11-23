#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct HorizontalLayout {
    pub component_id: u64,
    pub parent_component_id: u64,
}

impl HorizontalLayout {
    pub unsafe fn create_at_address(pointer: *mut u8, component_id: u64, parent_component_id: u64) -> usize {
        let object: *mut HorizontalLayout = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<HorizontalLayout>() as isize);

        // component_id
        (*object).component_id = component_id;

        // parent_component_id
        (*object).parent_component_id = parent_component_id;

        // return
        mem::size_of::<HorizontalLayout>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut HorizontalLayout, 1);
        let pointer = pointer.offset(mem::size_of::<HorizontalLayout>() as isize);

        // component_id

        // parent_component_id

        // return
        mem::size_of::<HorizontalLayout>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut HorizontalLayout = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<HorizontalLayout>() as isize);

        // component_id

        // parent_component_id

        // return
        (mem::size_of::<HorizontalLayout>(), object)
    }
}


