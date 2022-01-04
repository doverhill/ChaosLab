use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::{ Arc, Mutex };

pub const BOGUS_AUTO_GET_FILES_CLIENT_TO_SERVER_MESSAGE: u64 = 2;

pub const BOGUS_AUTO_GET_FILES_ARGUMENTS_OBJECT_ID: usize = 7;

#[derive(Default)]
pub struct GetFilesArguments {
    // fixed size fields
    // dynamically sized fields
    pub path: String
}

impl GetFilesArguments {
    const FIXED_SIZE: usize = ;

    pub fn new(path: &str) -> Self {
        GetFilesArguments {
            path: path.to_string()
        }
    }
}

impl ChannelObject for GetFilesArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write dynamically sized field path
        let length = self.path.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.path.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = GetFilesArguments::default();

        // read dynamically sized field path
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.path = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, path: &str) -> Result<crate::GetFilesFileInfoIterator, Error> {
    let channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = GetFilesArguments {
        path: path
    };
    channel.add_object(BOGUS_AUTO_GET_FILES_ARGUMENTS_OBJECT_ID, arguments);
    match channel.call_sync(BOGUS_AUTO_GET_FILES_CLIENT_TO_SERVER_MESSAGE, false, 1000) {
        Ok(()) => {
        },
        Err(error) => {
            Err(error)
        }
    }
}
