use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_SIMPLE_SUM_CLIENT_TO_SERVER_MESSAGE: u64 = 1;

pub const BOGUS_AUTO_SIMPLE_SUM_ARGUMENTS_OBJECT_ID: usize = 5;

#[derive(Default)]
pub struct SimpleSumArguments {
    // fixed size fields
    pub x: i32,
    pub y: i32
    // dynamically sized fields
}

impl SimpleSumArguments {
    const FIXED_SIZE: usize = mem::size_of::<i32>() + mem::size_of::<i32>();

    pub fn new(x: i32, y: i32) -> Self {
        SimpleSumArguments {
            x: x,
            y: y
        }
    }
}

impl ChannelObject for SimpleSumArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&SimpleSumArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SimpleSumArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&SimpleSumArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub const BOGUS_AUTO_SIMPLE_SUM_RESULT_OBJECT_ID: usize = 6;

#[derive(Default)]
pub struct SimpleSumResult {
    // fixed size fields
    pub result: i32
    // dynamically sized fields
}

impl SimpleSumResult {
    const FIXED_SIZE: usize = mem::size_of::<i32>();

    pub fn new(result: i32) -> Self {
        SimpleSumResult {
            result: result
        }
    }
}

impl ChannelObject for SimpleSumResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&SimpleSumResult, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SimpleSumResult::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&SimpleSumResult, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, x: i32, y: i32) -> Result<i32, Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = SimpleSumArguments::new(x, y);
    channel.add_object(BOGUS_AUTO_SIMPLE_SUM_ARGUMENTS_OBJECT_ID, arguments);
    match channel.call_sync(BOGUS_AUTO_SIMPLE_SUM_CLIENT_TO_SERVER_MESSAGE, false, 1000) {
        Ok(()) => {
            match channel.get_object::<SimpleSumResult>(0, BOGUS_AUTO_SIMPLE_SUM_RESULT_OBJECT_ID) {
                Ok(result) => {
                    Ok(result.result)
                },
                Err(error) => {
                    Err(error)
                }
            }
        },
        Err(error) => {
            Err(error)
        }
    }
}

pub fn handle(handler: &mut Box<dyn crate::BogusAutoServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<SimpleSumArguments>(0, BOGUS_AUTO_SIMPLE_SUM_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for SimpleSum: {:?}", error);
        }
    };

    let result = handler.simple_sum(arguments.x, arguments.y);

    channel.start();
    let response = SimpleSumResult::new(result);
    channel.add_object(BOGUS_AUTO_SIMPLE_SUM_RESULT_OBJECT_ID, response);
    channel.send(Channel::to_reply(BOGUS_AUTO_SIMPLE_SUM_CLIENT_TO_SERVER_MESSAGE, false));
}
