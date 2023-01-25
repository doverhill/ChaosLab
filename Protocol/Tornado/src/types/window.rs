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

pub struct Window {
    pub component_id: u64,
    pub parent_component_id: u64,
    pub title: String,
}

impl Window {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Window, 1);
        pointer = pointer.offset(mem::size_of::<Window>() as isize);

        mem::size_of::<Window>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        // String title
        let mut len = self.title.len();
        *(pointer as *mut usize) = len;
        core::ptr::copy(self.title.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Window>() + Self::reconstruct_at(object_pointer as *mut Window, object_pointer.offset(mem::size_of::<Window>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Window, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // U64 component_id

        // U64 parent_component_id

        // String title
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).title), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        size
    }
}


