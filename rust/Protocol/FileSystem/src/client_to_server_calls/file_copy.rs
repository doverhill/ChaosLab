use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::{ iter::Iterator, Arc, Mutex };

pub const FILE_SYSTEM_FILE_COPY_CLIENT_TO_SERVER_MESSAGE: u64 = 4;
use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const FILE_SYSTEM_FILE_COPY_ARGUMENTS_OBJECT_ID: usize = 5;

#[derive(Default)]
pub struct FileCopyArguments {
    // fixed size fields
    // dynamically sized fields
    pub source_full_path: String,
    pub target_full_path: String
}

impl FileCopyArguments {
    const FIXED_SIZE: usize = ;

    pub fn new(source_full_path: &str, target_full_path: &str) -> Self {
        FileCopyArguments {
            source_full_path: source_full_path.to_string(),
            target_full_path: target_full_path.to_string()
        }
    }
}

impl ChannelObject for FileCopyArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {

        // write dynamically sized field source_full_path
        let length = self.source_full_path.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.source_full_path.as_ptr(), pointer, length);
        let pointer = pointer.offset(length as isize);

        // write dynamically sized field target_full_path
        let length = self.target_full_path.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.target_full_path.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = FileCopyArguments::default();


        // read dynamically sized field source_full_path
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.source_full_path = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
        let pointer = pointer.offset(length as isize);

        // read dynamically sized field target_full_path
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.target_full_path = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

