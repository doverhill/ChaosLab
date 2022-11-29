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
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut OtherTypePathsEnum, 1);
        pointer = pointer.offset(mem::size_of::<OtherTypePathsEnum>() as isize);

        mem::size_of::<OtherTypePathsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;

        println!("writing OtherTypePathsEnum at {:p}", pointer);

        match self {
            OtherTypePathsEnum::TypeTestType(value) => {
                value.write_references_at(pointer)
            }
            OtherTypePathsEnum::TypeI64(value) => {
                0
            }
            OtherTypePathsEnum::TypeString(value) => {
                let mut len = value.len();
                *(pointer as *mut usize) = len;
                pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer as *mut u8, len);
                len = ((len + 7) / 8) * 8;
                mem::size_of::<usize>() + len
            }
        }
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut OtherTypePathsEnum,
        references_pointer: *mut u8,
    ) -> usize {
        let object = object_pointer as *mut OtherTypePathsEnumStruct;

        println!("reading OtherTypePathsEnum at {:p}", references_pointer);

        match ((*object).tag) {
            OtherTypePathsEnumStructTag::TypeTestType => {
                TestType::reconstruct_at(addr_of_mut!((*object).payload.payload_test_type) as *mut TestType, references_pointer)
            },
            OtherTypePathsEnumStructTag::TypeI64 => {
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
    }
}

pub struct OtherType {
    pub include: bool,
    pub offset: i64,
    pub paths: Vec<OtherTypePathsEnum>,
}

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

        // paths - rust will align for us
        println!("writing OtherType.paths at {:p}", pointer);
        let len = self.paths.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.paths.as_ptr(), pointer as *mut OtherTypePathsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherTypePathsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherTypePathsEnum>();

        for item in self.paths.iter() {
            println!("  writing OtherType.paths item at {:p}", pointer);
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

        // paths - rust will align for us
        println!("reading OtherType.paths from {:p}", pointer);
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
            println!("  reading OtherType.paths item from {:p}", references_pointer);
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
    pub other: OtherType,
    pub cities: Vec<String>,
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
        println!("writing TestType.name at {:p}", pointer);
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // write custom types
        println!("writing TestType.other at {:p}", pointer);
        let len = self.other.write_references_at(pointer);
        println!("===> {} bytes", len);
        pointer = pointer.offset(len as isize);
        size += len;

        // pointer is after strings, write arrays

        // objects - rust will align for us
        println!("writing TestType.objects at {:p}", pointer);
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut OtherType, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in self.objects.iter() {
            println!("  writing TestType.objects item at {:p}", pointer);
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        // cities
        println!("writing TestType.cities at {:p}", pointer);
        let len = self.cities.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.cities.as_ptr(), pointer as *mut String, len);
        pointer = pointer.offset(len as isize * mem::size_of::<String>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();

        for item in self.cities.iter() {
            println!("writing TestType.cities item at {:p}", pointer);
            let mut len = item.len();
            *(pointer as *mut usize) = len;
            pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, len);
            len = ((len + 7) / 8) * 8;
            pointer = pointer.offset(len as isize);
            size += mem::size_of::<usize>() + len;
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
        println!("reading TestType.name from {:p}", pointer);
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

        // read custom types
        println!("reading TestType.other from {:p}", pointer);
        let len = OtherType::reconstruct_at(addr_of_mut!((*object_pointer).other), pointer);
        println!("===> {} bytes", len);
        pointer = pointer.offset(len as isize);
        size += len;

        // pointer is after strings, read arrays

        // objects - rust will align for us
        println!("reading TestType.objects from {:p}", pointer);
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
            println!("  reading TestType.objects item from {:p}", references_pointer);
            let item_size =
                OtherType::reconstruct_at(pointer as *mut OtherType, references_pointer);
            pointer = pointer.offset(mem::size_of::<OtherType>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        // cities
        println!("reading TestType.cities from {:p}", pointer);
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign =
            ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut String, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).cities),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();

        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<String>() as isize);
        for item in (*object_pointer).cities.iter() {
            println!("  reading TestType.cities item from {:p}", references_pointer);
            let mut len = *(references_pointer as *const usize);
            references_pointer = references_pointer.offset(mem::size_of::<usize>() as isize);
            let mut assign = ManuallyDrop::new(String::from_raw_parts(references_pointer, len, len));
            core::ptr::write(pointer as *mut String, ManuallyDrop::take(&mut assign));
            pointer = pointer.offset(mem::size_of::<String>() as isize);
            len = ((len + 7) / 8) * 8;
            references_pointer = references_pointer.offset(len as isize);
            size += mem::size_of::<usize>() + len;
        }
        pointer = references_pointer;

        size
    }
}
