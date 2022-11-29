#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

pub struct PointerPressedParameters {
    pub position: Point,
    pub buttons: Vec<PointerButton>,
}

impl PointerPressedParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut PointerPressedParameters, 1);
        pointer = pointer.offset(mem::size_of::<PointerPressedParameters>() as isize);

        mem::size_of::<PointerPressedParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType position
        let len = self.position.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // Enum buttons
        let len = self.buttons.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.buttons.as_ptr(), pointer as *mut PointerButton, len);
        pointer = pointer.offset(len as isize * mem::size_of::<PointerButton>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<PointerButton>();

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<PointerPressedParameters>() + Self::reconstruct_at(object_pointer as *mut PointerPressedParameters, object_pointer.offset(mem::size_of::<PointerPressedParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut PointerPressedParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType position
        let len = Point::reconstruct_at(addr_of_mut!((*object_pointer).position), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // Enum buttons
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut PointerButton, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).buttons), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<PointerButton>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<PointerButton>() as isize);
        for item in (*object_pointer).buttons.iter() {
            let item_size = PointerButton::reconstruct_at(pointer as *mut PointerButton, references_pointer);
            pointer = pointer.offset(mem::size_of::<PointerButton>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



