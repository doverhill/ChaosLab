#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use alloc::vec::Vec;
use alloc::string::String;
use crate::types::*;
use crate::enums::*;

pub struct Table {
    pub name: String,
    pub description: String,
    pub columns: Vec<String>,
    pub rows: Vec<Map>,
}

impl Table {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut Table, 1);
        pointer = pointer.offset(mem::size_of::<Table>() as isize);

        mem::size_of::<Table>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // String name
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String description
        let mut len = self.description.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.description.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String columns
        let len = self.columns.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.columns.as_ptr(), pointer as *mut String, len);
        pointer = pointer.offset(len as isize * mem::size_of::<String>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();
        for item in self.columns.iter() {
            let mut len = item.len();
            *(pointer as *mut usize) = len;
            pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, len);
            len = ((len + 7) / 8) * 8;
            pointer = pointer.offset(len as isize);
            size += mem::size_of::<usize>() + len;
        }

        // CustomType rows
        let len = self.rows.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.rows.as_ptr(), pointer as *mut Map, len);
        pointer = pointer.offset(len as isize * mem::size_of::<Map>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<Map>();
        for item in self.rows.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<Table>() + Self::reconstruct_at(object_pointer as *mut Table, object_pointer.offset(mem::size_of::<Table>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut Table, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // String name
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).name), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String description
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).description), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // String columns
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut String, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).columns), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<String>() as isize);
        for item in (*object_pointer).columns.iter() {
            let mut len = *(references_pointer as *const usize);
            references_pointer = references_pointer.offset(mem::size_of::<usize>() as isize);
            let mut assign = ManuallyDrop::new(String::from_raw_parts(references_pointer, len, len));
            core::ptr::write(pointer as *mut String, ManuallyDrop::take(&mut assign));
            pointer = pointer.offset(mem::size_of::<String>() as isize);
            len = ((len + 7) / 8) * 8;
            references_pointer = references_pointer.offset(len as isize);
            size += mem::size_of::< usize > () + len;
        }
        pointer = references_pointer;

        // CustomType rows
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut Map, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).rows), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<Map>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<Map>() as isize);
        for item in (*object_pointer).rows.iter() {
            let item_size = Map::reconstruct_at(pointer as *mut Map, references_pointer);
            pointer = pointer.offset(mem::size_of::<Map>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



