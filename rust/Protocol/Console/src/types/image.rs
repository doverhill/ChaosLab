#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

pub struct Image {
    pub size: Size,
    pub pixels: Vec<Color>,
}

impl Image {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Image, 1);
        pointer = pointer.offset(mem::size_of::<Image>() as isize);

        mem::size_of::<Image>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // CustomType size
        let len = self.size.write_references_at(pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType pixels
        let len = self.pixels.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.pixels.as_ptr(), pointer as *mut Color, len);
        pointer = pointer.offset(len as isize * mem::size_of::<Color>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<Color>();
        for item in self.pixels.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Image>() + Self::reconstruct_at(object_pointer as *mut Image, object_pointer.offset(mem::size_of::<Image>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Image, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // CustomType size
        let len = Size::reconstruct_at(addr_of_mut!((*object_pointer).size), pointer);
        pointer = pointer.offset(len as isize);
        size += len;

        // CustomType pixels
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut Color, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).pixels), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<Color>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<Color>() as isize);
        for item in (*object_pointer).pixels.iter() {
            let item_size = Color::reconstruct_at(pointer as *mut Color, references_pointer);
            pointer = pointer.offset(mem::size_of::<Color>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



