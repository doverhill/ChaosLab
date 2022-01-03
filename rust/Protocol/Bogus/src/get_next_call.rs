use library_chaos::{ Error, Channel, ChannelObject };
use std::mem;
use std::iter::Iterator;
use std::sync::{ Arc, Mutex };
use crate::server::BogusServerImplementation;
use crate::types::FileInfo;

pub const BOGUS_GET_NEXT_RESULT_OBJECT_ID: usize = 2;
#[derive(Default)]
struct GetNextResult {
    result: usize
}

impl ChannelObject for GetNextResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        *(pointer as *mut GetNextResult) = self;
        mem::size_of::<GetNextResult>()
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = GetNextResult::default();
        core::ptr::copy(pointer as *mut GetNextResult, &mut object, 1);
        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>) -> Result<usize, Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    
    match channel.call_sync(crate::client::BOGUS_GET_NEXT_CLIENT_MESSAGE, false, 1000) {
        Ok(()) => {
            match channel.get_object::<GetNextResult>(0, BOGUS_GET_NEXT_RESULT_OBJECT_ID) {
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

    let result = handler.get_next();

    // FIXME detect when channel is full and send partial result using has_more flag
    channel.start();
    let response = GetNextResult {
        result: result
    };
    channel.add_object(BOGUS_GET_NEXT_RESULT_OBJECT_ID, response);
    channel.send(Channel::to_reply(crate::client::BOGUS_GET_NEXT_CLIENT_MESSAGE, false));
}