use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_SET_TEXT_CURSOR_POSITION_CLIENT_TO_SERVER_MESSAGE: u64 = 3;

pub const CONSOLE_SET_TEXT_CURSOR_POSITION_ARGUMENTS_OBJECT_ID: usize = 5;

#[derive(Default)]
pub struct SetTextCursorPositionArguments {
    // fixed size fields
    pub column: usize,
    pub row: usize
    // dynamically sized fields
}

impl SetTextCursorPositionArguments {
    const FIXED_SIZE: usize = mem::size_of::<usize>() + mem::size_of::<usize>();

    pub fn new(column: usize, row: usize) -> Self {
        SetTextCursorPositionArguments {
            column: column,
            row: row
        }
    }
}

impl ChannelObject for SetTextCursorPositionArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&SetTextCursorPositionArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SetTextCursorPositionArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&SetTextCursorPositionArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, column: usize, row: usize) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = SetTextCursorPositionArguments::new(column, row);
    channel.add_object(CONSOLE_SET_TEXT_CURSOR_POSITION_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_SET_TEXT_CURSOR_POSITION_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<SetTextCursorPositionArguments>(0, CONSOLE_SET_TEXT_CURSOR_POSITION_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for SetTextCursorPosition: {:?}", error);
        }
    };

    let result = handler.set_text_cursor_position(arguments.column, arguments.row);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_SET_TEXT_CURSOR_POSITION_CLIENT_TO_SERVER_MESSAGE, false));
}
