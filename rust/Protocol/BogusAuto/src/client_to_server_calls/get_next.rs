use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::{ Arc, Mutex };

pub const BOGUS_AUTO_GET_NEXT_CLIENT_TO_SERVER_MESSAGE: u64 = 4;

pub const BOGUS_AUTO_GET_NEXT_RESULT_OBJECT_ID: usize = 9;

#[derive(Default)]
pub struct GetNextResult {
    // fixed size fields
    pub result: usize
    // dynamically sized fields
}

impl GetNextResult {
    const FIXED_SIZE: usize = mem::size_of::<usize>();

    pub fn new(result: usize) -> Self {
        GetNextResult {
            result: result
        }
    }
}

impl ChannelObject for GetNextResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&GetNextResult, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = GetNextResult::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&GetNextResult, *mut u8>(&object), Self::FIXED_SIZE);
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, ) -> Result<usize, Error> {
    let channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = GetNextArguments {
    };
    channel.add_object(BOGUS_AUTO_GET_NEXT_ARGUMENTS_OBJECT_ID, arguments);
    match channel.call_sync(BOGUS_AUTO_GET_NEXT_CLIENT_TO_SERVER_MESSAGE, false, 1000) {
        Ok(()) => {
        },
        Err(error) => {
            Err(error)
        }
    }
}
