use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_GET_CAPABILITIES_CLIENT_TO_SERVER_MESSAGE: u64 = 1;

pub const CONSOLE_GET_CAPABILITIES_RESULT_OBJECT_ID: usize = 3;

#[derive(Default)]
pub struct GetCapabilitiesResult {
    // fixed size fields
    pub is_framebuffer: bool,
    pub framebuffer_width: usize,
    pub framebuffer_height: usize,
    pub text_columns: usize,
    pub text_rows: usize
    // dynamically sized fields
}

impl GetCapabilitiesResult {
    const FIXED_SIZE: usize = mem::size_of::<bool>() + mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<usize>();

    pub fn new(is_framebuffer: bool, framebuffer_width: usize, framebuffer_height: usize, text_columns: usize, text_rows: usize) -> Self {
        GetCapabilitiesResult {
            is_framebuffer: is_framebuffer,
            framebuffer_width: framebuffer_width,
            framebuffer_height: framebuffer_height,
            text_columns: text_columns,
            text_rows: text_rows
        }
    }
}

impl ChannelObject for GetCapabilitiesResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&GetCapabilitiesResult, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = GetCapabilitiesResult::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&GetCapabilitiesResult, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, ) -> Result<GetCapabilitiesResult, Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    match channel.call_sync(CONSOLE_GET_CAPABILITIES_CLIENT_TO_SERVER_MESSAGE, false, 1000) {
        Ok(()) => {
            match channel.get_object::<GetCapabilitiesResult>(0, CONSOLE_GET_CAPABILITIES_RESULT_OBJECT_ID) {
                Ok(result) => {
                    Ok(result)
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

pub fn handle(handler: &mut Box<dyn crate::ConsoleServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let result = handler.get_capabilities();

    channel.start();
    let response = GetCapabilitiesResult::new(result.is_framebuffer, result.framebuffer_width, result.framebuffer_height, result.text_columns, result.text_rows);
    channel.add_object(CONSOLE_GET_CAPABILITIES_RESULT_OBJECT_ID, response);
    channel.send(Channel::to_reply(CONSOLE_GET_CAPABILITIES_CLIENT_TO_SERVER_MESSAGE, false));
}
