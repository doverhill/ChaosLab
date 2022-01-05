use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_KEY_RELEASED_SERVER_TO_CLIENT_MESSAGE: u64 = 7;

pub const CONSOLE_KEY_RELEASED_ARGUMENTS_OBJECT_ID: usize = 8;

#[derive(Default)]
pub struct KeyReleasedArguments {
    // fixed size fields
    pub key_code: usize
    // dynamically sized fields
}

impl KeyReleasedArguments {
    const FIXED_SIZE: usize = mem::size_of::<usize>();

    pub fn new(key_code: usize) -> Self {
        KeyReleasedArguments {
            key_code: key_code
        }
    }
}

impl ChannelObject for KeyReleasedArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&KeyReleasedArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = KeyReleasedArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&KeyReleasedArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, key_code: usize) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = KeyReleasedArguments::new(key_code);
    channel.add_object(CONSOLE_KEY_RELEASED_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_KEY_RELEASED_SERVER_TO_CLIENT_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleClientImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<KeyReleasedArguments>(0, CONSOLE_KEY_RELEASED_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for KeyReleased: {:?}", error);
        }
    };

    let result = handler.key_released(arguments.key_code);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_KEY_RELEASED_SERVER_TO_CLIENT_MESSAGE, false));
}
