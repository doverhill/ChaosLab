use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const CONSOLE_COLOR_OBJECT_ID: usize = 1;

#[derive(Default)]
pub struct Color {
    // fixed size fields
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
    // dynamically sized fields
}

impl Color {
    const FIXED_SIZE: usize = mem::size_of::<u8>() + mem::size_of::<u8>() + mem::size_of::<u8>() + mem::size_of::<u8>();

    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red: red,
            green: green,
            blue: blue,
            alpha: alpha
        }
    }
}

impl ChannelObject for Color {
    fn get_size() -> usize {
        Self::FIXED_SIZE
    }

    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&Color, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = Color::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&Color, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

