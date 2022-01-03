use library_chaos::{ Error, Channel, ChannelObject };
use crate::server::BogusServerImplementation;
use std::sync::{ Arc, Mutex };
use std::mem;

pub const BOGUS_SIMPLE_SUM_ARGUMENTS_OBJECT_ID: usize = 1;
#[derive(Default)]
pub struct SimpleSumArguments {
    pub x: i32,
    pub y: i32
}

impl ChannelObject for SimpleSumArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        *(pointer as *mut SimpleSumArguments) = self;

        mem::size_of::<SimpleSumArguments>()
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SimpleSumArguments::default();
        core::ptr::copy(pointer as *mut SimpleSumArguments, &mut object, 1);
        object
    }
}

pub const BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID: usize = 2;
#[derive(Default)]
struct SimpleSumResult {
    result: i32
}

impl ChannelObject for SimpleSumResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        *(pointer as *mut SimpleSumResult) = self;

        mem::size_of::<SimpleSumResult>()
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SimpleSumResult::default();
        core::ptr::copy(pointer as *mut SimpleSumResult, &mut object, 1);
        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, x: i32, y: i32) -> Result<i32, Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = SimpleSumArguments {
        x: x,
        y: y
    };
    channel.add_object(BOGUS_SIMPLE_SUM_ARGUMENTS_OBJECT_ID, arguments);
    
    match channel.call_sync(crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE, false, 1000) {
        Ok(()) => {
            match channel.get_object::<SimpleSumResult>(0, BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID) {
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

pub fn handle(handler: &mut Box<dyn BogusServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();

    let arguments = match channel.get_object::<SimpleSumArguments>(0, BOGUS_SIMPLE_SUM_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments: {:?}", error);
        }
    };

    let result = handler.simple_sum(arguments.x, arguments.y);
    channel.start();
    let response = SimpleSumResult {
        result: result
    };
    channel.add_object(BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID, response);
    channel.send(Channel::to_reply(crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE, false));
}