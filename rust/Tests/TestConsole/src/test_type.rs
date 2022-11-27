use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};
use core::ptr::addr_of_mut;

// OtherType IDL
// paths: TestType|i64|string[]
// include: bool
// offset: i64

// pub enum OtherTypePathsEnum {
//     TypeTestType(TestType),
//     TypeI64(i64),
//     TypeString(String)
// }

pub struct OtherType {
    pub include: bool,
    pub offset: i64,
    // paths: Vec<OtherTypePathsEnum>,
}

// pub struct OtherTypeFixed {
//     include: bool,
//     offset: i64,
// }

// pub struct OtherTypeRead {
//     include: bool,
//     offset: i64,
//     paths: Vec<*const OtherTypePathsEnum>,
// }

impl OtherType {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut OtherType, 1);
        pointer = pointer.offset(mem::size_of::<OtherType>() as isize);

        mem::size_of::<OtherType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        0
    }

    pub unsafe fn reconstruct_at(pointer: *mut u8) -> usize {
        0
    }
}

// TestType IDL
// name: string
// size: u64
// objects: OtherType[]

pub struct TestType {
    pub size: u64,
    pub name: String,
    pub objects: Vec<OtherType>,
}

impl TestType {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut TestType, 1);
        pointer = pointer.offset(mem::size_of::<TestType>() as isize);

        mem::size_of::<TestType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // pointer is after TestType, write strings

        // name - make sure to 8 byte align pointer after writing
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // pointer is after strings, write arrays

        // objects - rust will align for us
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut OtherType, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in self.objects.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at(pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let object = pointer as *mut TestType;
        pointer = pointer.offset(mem::size_of::<TestType>() as isize);
        let mut size = mem::size_of::<TestType>();

        // pointer is after TestType, read strings

        // name - make sure to 8 byte align pointer after reading
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(addr_of_mut!((*object).name), ManuallyDrop::take(&mut assign));
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // pointer is after strings, read arrays

        // objects - rust will align for us
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut OtherType, len, len));
        core::ptr::write(addr_of_mut!((*object).objects), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in (*object).objects.iter() {
            let item_size = OtherType::reconstruct_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }
}
