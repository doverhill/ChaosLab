use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_NOTIFY_SERVER_TO_CLIENT_MESSAGE: u64 = 6;

pub const BOGUS_AUTO_NOTIFY_ARGUMENTS_OBJECT_ID: usize = 12;

#[derive(Default)]
pub struct NotifyArguments {
    // fixed size fields
    // dynamically sized fields
    pub message: String
}

impl NotifyArguments {
    pub fn new(message: &str) -> Self {
        NotifyArguments {
            message: message.to_string()
        }
    }
}

impl ChannelObject for NotifyArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write dynamically sized field message
        let message_length = self.message.len();
        *(pointer as *mut usize) = message_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        ptr::copy(self.message.as_ptr(), pointer, message_length);

        mem::size_of::<usize>() + message_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = NotifyArguments::default();

        // read dynamically sized field message
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.message = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, message: &str) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = NotifyArguments::new(message);
    channel.add_object(BOGUS_AUTO_NOTIFY_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(BOGUS_AUTO_NOTIFY_SERVER_TO_CLIENT_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::BogusAutoClientImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<NotifyArguments>(0, BOGUS_AUTO_NOTIFY_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for Notify: {:?}", error);
        }
    };

    let result = handler.notify(&arguments.message);

    channel.start();
    channel.send(Channel::to_reply(BOGUS_AUTO_NOTIFY_SERVER_TO_CLIENT_MESSAGE, false));
}
