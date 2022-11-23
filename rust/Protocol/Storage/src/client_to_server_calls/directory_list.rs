use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::{ iter::Iterator, Arc, Mutex };

pub const FILE_SYSTEM_DIRECTORY_LIST_CLIENT_TO_SERVER_MESSAGE: u64 = 1;
use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const FILE_SYSTEM_DIRECTORY_LIST_ARGUMENTS_OBJECT_ID: usize = 2;

#[derive(Default)]
pub struct DirectoryListArguments {
    // fixed size fields
    // dynamically sized fields
    pub full_path: String
}

impl DirectoryListArguments {
    const FIXED_SIZE: usize = ;

    pub fn new(full_path: &str) -> Self {
        DirectoryListArguments {
            full_path: full_path.to_string()
        }
    }
}

impl ChannelObject for DirectoryListArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {

        // write dynamically sized field full_path
        let length = self.full_path.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.full_path.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = DirectoryListArguments::default();


        // read dynamically sized field full_path
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.full_path = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

