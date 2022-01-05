use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_SET_TEXT_COLOR_CLIENT_TO_SERVER_MESSAGE: u64 = 2;

pub const CONSOLE_SET_TEXT_COLOR_ARGUMENTS_OBJECT_ID: usize = 4;

#[derive(Default)]
pub struct SetTextColorArguments {
    // fixed size fields
    pub color: Color,
    pub background_color: Color
    // dynamically sized fields
}

impl SetTextColorArguments {
    const FIXED_SIZE: usize = mem::size_of::<Color>() + mem::size_of::<Color>();

    pub fn new(color: Color, background_color: Color) -> Self {
        SetTextColorArguments {
            color: color,
            background_color: background_color
        }
    }
}

impl ChannelObject for SetTextColorArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&SetTextColorArguments, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = SetTextColorArguments::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&SetTextColorArguments, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, color: Color, background_color: Color) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = SetTextColorArguments::new(color, background_color);
    channel.add_object(CONSOLE_SET_TEXT_COLOR_ARGUMENTS_OBJECT_ID, arguments);
    channel.call_sync(CONSOLE_SET_TEXT_COLOR_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    let arguments = match channel.get_object::<SetTextColorArguments>(0, CONSOLE_SET_TEXT_COLOR_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments for SetTextColor: {:?}", error);
        }
    };

    let result = handler.set_text_color(arguments.color, arguments.background_color);

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_SET_TEXT_COLOR_CLIENT_TO_SERVER_MESSAGE, false));
}
