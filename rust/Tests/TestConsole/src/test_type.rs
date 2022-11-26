use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

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
        core::ptr::copy(self, pointer as *mut OtherType, 1);
        let pointer = pointer.offset(mem::size_of::<OtherType>() as isize);

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
        core::ptr::copy(self, pointer as *mut TestType, 1);
        let pointer = pointer.offset(mem::size_of::<TestType>() as isize);

        mem::size_of::<TestType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut size: usize = 0;

        // pointer is after TestType, write strings

        // name - make sure to 8 byte align pointer after writing
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        let pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // pointer is after strings, write arrays

        // objects - rust will align for us
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut OtherType, len);
        let pointer = pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in self.objects.iter() {
            let item_size = item.write_references_at(pointer);
            let pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at(pointer: *mut u8) -> usize {
        let object = pointer as *mut TestType;
        let pointer = pointer.offset(mem::size_of::<TestType>() as isize);
        let mut size = mem::size_of::<TestType>();

        // pointer is after TestType, read strings

        // name - make sure to 8 byte align pointer after reading
        let mut len = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).name = String::from_raw_parts(pointer, len, len);
        len = ((len + 7) / 8) * 8;
        let pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // pointer is after strings, read arrays

        // objects - rust will align for us
        let len = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        (*object).objects = Vec::from_raw_parts(pointer as *mut OtherType, len, len);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in (*object).objects.iter() {
            let item_size = OtherType::reconstruct_at(pointer);
            let pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }
}

// pub struct TestTypeRead {
//     size: u64,
//     name: String,
//     objects: Vec<*const OtherType>,
// }