use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const BOGUS_AUTO_BUTTON_OBJECT_ID: usize = 4;

#[derive(Default)]
pub struct Button {
    // fixed size fields
    pub component_id: u64,
    pub parent_component_id: u64
    // dynamically sized fields
    pub icon_name: String,
    pub text: String
}

impl Button {
    const FIXED_SIZE: usize = mem::size_of::<u64>() + mem::size_of::<u64>();

    pub fn new(component_id: u64, parent_component_id: u64, icon_name: &str, text: &str) -> Self {
        Button {
            component_id: component_id,
            parent_component_id: parent_component_id,
            icon_name: icon_name.to_string(),
            text: text.to_string()
        }
    }
}

impl ChannelObject for Button {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&Button, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamically sized field icon_name
        let length = self.icon_name.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.icon_name.as_ptr(), pointer, length);
        let pointer = pointer.offset(length as isize);

        // write dynamically sized field text
        let length = self.text.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.text.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = Button::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&Button, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamically sized field icon_name
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.icon_name = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
        let pointer = pointer.offset(length as isize);

        // read dynamically sized field text
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.text = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

