use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const BOGUS_AUTO_WINDOW_OBJECT_ID: usize = 3;

#[derive(Default)]
pub struct Window {
    // fixed size fields
    pub component_id: u64,
    pub parent_component_id: u64
    // dynamically sized fields
    pub title: String
}

impl Window {
    const FIXED_SIZE: usize = mem::size_of::<u64>() + mem::size_of::<u64>();

    pub fn new(component_id: u64, parent_component_id: u64, title: &str) -> Self {
        Window {
            component_id: component_id,
            parent_component_id: parent_component_id,
            title: title.to_string()
        }
    }
}

impl ChannelObject for Window {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&Window, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamically sized field title
        let length = self.title.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.title.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = Window::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&Window, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamically sized field title
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.title = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

