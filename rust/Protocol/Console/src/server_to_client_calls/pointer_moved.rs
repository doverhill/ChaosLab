use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_POINTER_MOVED_SERVER_TO_CLIENT_MESSAGE: u64 = 9;

pub const CONSOLE_POINTER_MOVED_ARGUMENTS_OBJECT_ID: usize = 10;

#[derive(Default)]
pub struct PointerMovedArguments {
    // fixed size fields
    pub x: usize,
    pub y: usize
    // dynamically sized fields
}

impl PointerMovedArguments {
    const FIXED_SIZE: usize = mem::size_of::<usize>() + mem::size_of::<usize>();

    pub fn new(x: usize, y: usize) -> Self {
        PointerMovedArguments {
            x: x,
            y: y
        }
    }
}

impl ChannelObject for PointerMovedArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&PointerMovedArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = PointerMovedArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&PointerMovedArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, x: usize, y: usize) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = PointerMovedArguments::new(x, y);
    channel.add_object(CONSOLE_POINTER_MOVED_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_POINTER_MOVED_SERVER_TO_CLIENT_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleClientImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<PointerMovedArguments>(0, CONSOLE_POINTER_MOVED_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for PointerMoved: {:?}", error);
        }
    };

    let result = handler.pointer_moved(arguments.x, arguments.y);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_POINTER_MOVED_SERVER_TO_CLIENT_MESSAGE, false));
}
