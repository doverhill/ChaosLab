use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_WRITE_TEXT_CLIENT_TO_SERVER_MESSAGE: u64 = 4;

pub const CONSOLE_WRITE_TEXT_ARGUMENTS_OBJECT_ID: usize = 6;

#[derive(Default)]
pub struct WriteTextArguments {
    // fixed size fields
    // dynamically sized fields
    pub text: String
}

impl WriteTextArguments {
    pub fn new(text: &str) -> Self {
        WriteTextArguments {
            text: text.to_string()
        }
    }
}

impl ChannelObject for WriteTextArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write dynamically sized field text
        let text_length = self.text.len();
        *(pointer as *mut usize) = text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        ptr::copy(self.text.as_ptr(), pointer, text_length);

        mem::size_of::<usize>() + text_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = WriteTextArguments::default();

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
    let arguments = WriteTextArguments::new(text);
    channel.add_object(CONSOLE_WRITE_TEXT_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_WRITE_TEXT_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<WriteTextArguments>(0, CONSOLE_WRITE_TEXT_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for WriteText: {:?}", error);
        }
    };

    let result = handler.write_text(&arguments.text);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_WRITE_TEXT_CLIENT_TO_SERVER_MESSAGE, false));
}
