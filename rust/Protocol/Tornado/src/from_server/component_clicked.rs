#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct ComponentClickedParameters {
    pub component_id: u64,
}

impl ComponentClickedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, component_id: u64) -> usize {
        let object: *mut ComponentClickedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ComponentClickedParameters>() as isize);

        // component_id
        (*object).component_id = component_id;

        // return
        mem::size_of::<ComponentClickedParameters>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut ComponentClickedParameters, 1);
        let pointer = pointer.offset(mem::size_of::<ComponentClickedParameters>() as isize);

        // component_id

        // return
        mem::size_of::<ComponentClickedParameters>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut ComponentClickedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<ComponentClickedParameters>() as isize);

        // component_id

        // return
        (mem::size_of::<ComponentClickedParameters>(), object)
    }
}


