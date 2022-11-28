use core::ptr::addr_of_mut;
use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

// OtherType IDL
// paths: TestType|i64|string[]
// include: bool
// offset: i64

#[repr(C, u64)]
pub enum OtherTypePathsEnum {
    TypeTestType(TestType),
    TypeI64(i64),
    TypeString(String),
}

#[repr(C)]
struct OtherTypePathsEnumStruct {
    tag: OtherTypePathsEnumStructTag,
    payload: OtherTypePathsEnumStructPayload,
}

#[repr(u64)]
enum OtherTypePathsEnumStructTag {
    TypeTestType,
    TypeI64,
    TypeString,
}

#[repr(C)]
union OtherTypePathsEnumStructPayload {
    payload_test_type: ManuallyDrop<TestType>,
    payload_i64: i64,
    payload_string: ManuallyDrop<String>,
}

impl OtherTypePathsEnum {
    // pub const OPTION_TESTTYPE: usize = 1;
    // pub const OPTION_I64: usize = 1;
    // pub const OPTION_STRING: usize = 1;

    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut OtherTypePathsEnum, 1);
        pointer = pointer.offset(mem::size_of::<OtherTypePathsEnum>() as isize);

        mem::size_of::<OtherTypePathsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;

        // let object = pointer;
        // let mut pointer = pointer.offset(mem::size_of::<usize>() as isize);

        match self {
            OtherTypePathsEnum::TypeTestType(value) => {
                // *(object as *mut usize) = Self::OPTION_TESTTYPE;
                value.write_at(pointer)
                // (value.as_ref().unwrap()).write_at_address(pointer)
            }
            OtherTypePathsEnum::TypeI64(value) => {
                // *(object as *mut usize) = Self::OPTION_I64;
                // *(pointer as *mut i64) = *value;
                // mem::size_of::<usize>()
                // (value.as_ref().unwrap()).write_at_address(pointer)
                0
            }
            OtherTypePathsEnum::TypeString(value) => {
                // *(object as *mut usize) = Self::OPTION_STRING;
                let mut len = value.len();
                *(pointer as *mut usize) = len;
                pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer as *mut u8, len);
                len = ((len + 7) / 8) * 8;
                mem::size_of::<usize>() + len
                // (value.as_ref().unwrap()).write_at_address(pointer)
            }
        }
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut OtherTypePathsEnum,
        references_pointer: *mut u8,
    ) -> usize {
        let object = object_pointer as *mut OtherTypePathsEnumStruct;
        // let enum_type = (*pointer as usize);
        // pointer = pointer.offset(mem::size_of::<usize>() as isize);

        match ((*object).tag) {
            OtherTypePathsEnumStructTag::TypeTestType => {
                TestType::reconstruct_at(addr_of_mut!((*object).payload.payload_string) as *mut TestType, references_pointer)
                // core::ptr::write(addr_of_mut!((*object).payload.payload_test_type), pointer as *const TestType);
                // let dst = &mut ManuallyDrop::take(&mut (*object).payload.payload_test_type) as *mut TestType;
                // core::ptr::write(dst, pointer as *const TestType);
                // size
            },
            OtherTypePathsEnumStructTag::TypeI64 => {
                // *(object).payload.payload_i64 = *(pointer as *const i64);
                // let inner = *(pointer as *const i64);
                // let mut value = ManuallyDrop::new(Self::TypeI64(inner));
                // core::ptr::write(object_pointer, ManuallyDrop::take(&mut value));
                // mem::size_of::<usize>()
                0
            },
            OtherTypePathsEnumStructTag::TypeString => {
                let mut pointer = references_pointer;
                let mut len = *(pointer as *const usize);
                pointer = pointer.offset(mem::size_of::<usize>() as isize);
                (*object).payload.payload_string = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
                len = ((len + 7) / 8) * 8;
                mem::size_of::<usize>() + len
            },
        }

        // let size = match enum_type {
        //     Self::OPTION_TESTTYPE => {
        //         let size = TestType::reconstruct_at_inline(references_pointer);
        //         *object_pointer = Self::TypeTestType(*references_pointer)
        //     },
        //     Self::OPTION_I64 => {

        //     },
        //     Self::OPTION_STRING => {

        //     },
        // };

        // mem::size_of::<usize>() + size
    }
}

pub struct OtherType {
    pub include: bool,
    pub offset: i64,
    pub paths: Vec<OtherTypePathsEnum>,
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
        let mut pointer = pointer;
        let mut size: usize = 0;

        // pointer is after TestType, write strings

        // pointer is after strings, write arrays

        // objects - rust will align for us
        let len = self.paths.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.paths.as_ptr(), pointer as *mut OtherTypePathsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherTypePathsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherTypePathsEnum>();

        for item in self.paths.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        let size = Self::reconstruct_at(
            object_pointer as *mut OtherType,
            object_pointer.offset(mem::size_of::<OtherType>() as isize),
        );
        mem::size_of::<OtherType>() + size
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut OtherType,
        references_pointer: *mut u8,
    ) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // pointer is after TestType, read strings

        // pointer is after strings, read arrays

        // objects - rust will align for us
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(
            pointer as *mut OtherTypePathsEnum,
            len,
            len,
        ));
        core::ptr::write(
            addr_of_mut!((*object_pointer).paths),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherTypePathsEnum>();

        // pointer is at first item in array
        // set references pointer to after array
        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<OtherTypePathsEnum>() as isize);
        for item in (*object_pointer).paths.iter() {
            let item_size = OtherTypePathsEnum::reconstruct_at(
                pointer as *mut OtherTypePathsEnum,
                references_pointer,
            );
            pointer = pointer.offset(mem::size_of::<OtherTypePathsEnum>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
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
        println!("WRITE_AT TestType {:p}", pointer);
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut TestType, 1);
        pointer = pointer.offset(mem::size_of::<TestType>() as isize);

        mem::size_of::<TestType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        println!("WRITE_REFERENCES_AT TestType {:p}", pointer);
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

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        let size = Self::reconstruct_at(
            object_pointer as *mut TestType,
            object_pointer.offset(mem::size_of::<TestType>() as isize),
        );
        mem::size_of::<TestType>() + size
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut TestType,
        references_pointer: *mut u8,
    ) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // pointer is after TestType, read strings

        // name - make sure to 8 byte align pointer after reading
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).name),
            ManuallyDrop::take(&mut assign),
        );
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // pointer is after strings, read arrays

        // objects - rust will align for us
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign =
            ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut OtherType, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).objects),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        // pointer is at first item in array
        // set references pointer to after array
        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        for item in (*object_pointer).objects.iter() {
            let item_size =
                OtherType::reconstruct_at(pointer as *mut OtherType, references_pointer);
            pointer = pointer.offset(mem::size_of::<OtherType>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}
