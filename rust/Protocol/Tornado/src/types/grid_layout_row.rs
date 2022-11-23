#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct GridLayoutRow {
    pub component_id: u64,
    pub parent_component_id: u64,
    pub size_mode: SizeMode,
    pub fraction: u64,
}

impl GridLayoutRow {
    pub unsafe fn create_at_address(pointer: *mut u8, component_id: u64, parent_component_id: u64, size_mode: SizeMode, fraction: u64) -> usize {
        let object: *mut GridLayoutRow = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<GridLayoutRow>() as isize);

        // component_id
        (*object).component_id = component_id;

        // parent_component_id
        (*object).parent_component_id = parent_component_id;

        // size_mode

        // fraction
        (*object).fraction = fraction;

        // return
        mem::size_of::<GridLayoutRow>()
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut GridLayoutRow, 1);
        let pointer = pointer.offset(mem::size_of::<GridLayoutRow>() as isize);

        // component_id

        // parent_component_id

        // size_mode

        // fraction

        // return
        mem::size_of::<GridLayoutRow>()
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut GridLayoutRow = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<GridLayoutRow>() as isize);

        // component_id

        // parent_component_id

        // size_mode

        // fraction

        // return
        (mem::size_of::<GridLayoutRow>(), object)
    }
}


