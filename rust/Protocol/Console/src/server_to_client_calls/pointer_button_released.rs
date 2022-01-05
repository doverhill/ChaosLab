use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_POINTER_BUTTON_RELEASED_SERVER_TO_CLIENT_MESSAGE: u64 = 11;

pub const CONSOLE_POINTER_BUTTON_RELEASED_ARGUMENTS_OBJECT_ID: usize = 12;

#[derive(Default)]
pub struct PointerButtonReleasedArguments {
    // fixed size fields
    pub x: usize,
    pub y: usize,
    pub button_number: usize
    // dynamically sized fields
}

impl PointerButtonReleasedArguments {
    const FIXED_SIZE: usize = mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<usize>();

    pub fn new(x: usize, y: usize, button_number: usize) -> Self {
        PointerButtonReleasedArguments {
            x: x,
            y: y,
            button_number: button_number
        }
    }
}

impl ChannelObject for PointerButtonReleasedArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&PointerButtonReleasedArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = PointerButtonReleasedArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&PointerButtonReleasedArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, x: usize, y: usize, button_number: usize) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = PointerButtonReleasedArguments::new(x, y, button_number);
    channel.add_object(CONSOLE_POINTER_BUTTON_RELEASED_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_POINTER_BUTTON_RELEASED_SERVER_TO_CLIENT_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleClientImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<PointerButtonReleasedArguments>(0, CONSOLE_POINTER_BUTTON_RELEASED_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for PointerButtonReleased: {:?}", error);
        }
    };

    let result = handler.pointer_button_released(arguments.x, arguments.y, arguments.button_number);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_POINTER_BUTTON_RELEASED_SERVER_TO_CLIENT_MESSAGE, false));
}
