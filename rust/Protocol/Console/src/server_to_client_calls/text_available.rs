use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_TEXT_AVAILABLE_SERVER_TO_CLIENT_MESSAGE: u64 = 8;

pub const CONSOLE_TEXT_AVAILABLE_ARGUMENTS_OBJECT_ID: usize = 9;

#[derive(Default)]
pub struct TextAvailableArguments {
    // fixed size fields
    // dynamically sized fields
    pub text: String
}

impl TextAvailableArguments {
    pub fn new(text: &str) -> Self {
        TextAvailableArguments {
            text: text.to_string()
        }
    }
}

impl ChannelObject for TextAvailableArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write dynamically sized field text
        let text_length = self.text.len();
        *(pointer as *mut usize) = text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        ptr::copy(self.text.as_ptr(), pointer, text_length);

        mem::size_of::<usize>() + text_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = TextAvailableArguments::default();

        // read dynamically sized field text
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.text = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, text: &str) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = TextAvailableArguments::new(text);
    channel.add_object(CONSOLE_TEXT_AVAILABLE_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_TEXT_AVAILABLE_SERVER_TO_CLIENT_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleClientImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<TextAvailableArguments>(0, CONSOLE_TEXT_AVAILABLE_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for TextAvailable: {:?}", error);
        }
    };

    let result = handler.text_available(&arguments.text);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_TEXT_AVAILABLE_SERVER_TO_CLIENT_MESSAGE, false));
}
