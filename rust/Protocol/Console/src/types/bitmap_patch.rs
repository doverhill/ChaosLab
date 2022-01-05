use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const CONSOLE_BITMAP_PATCH_OBJECT_ID: usize = 2;

#[derive(Default)]
pub struct BitmapPatch {
    // fixed size fields
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub data: Color
    // dynamically sized fields
}

impl BitmapPatch {
    const FIXED_SIZE: usize = mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<usize>() + mem::size_of::<Color>();

    pub fn new(x: usize, y: usize, width: usize, height: usize, data: Color) -> Self {
        BitmapPatch {
            x: x,
            y: y,
            width: width,
            height: height,
            data: data
        }
    }
}

impl ChannelObject for BitmapPatch {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&BitmapPatch, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);

        Self::FIXED_SIZE
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = BitmapPatch::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&BitmapPatch, *mut u8>(&object), Self::FIXED_SIZE);

        object
    }
}

