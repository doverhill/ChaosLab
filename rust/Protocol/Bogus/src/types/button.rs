use library_chaos::ChannelObject;
use core::mem;
use core::ptr;

pub const BOGUS_TYPE_BUTTON_OBJECT_ID: usize = 8;
#[derive(Default)]
pub struct Button {
    pub base: crate::Component,
    pub icon_name: String,
    pub text: String
}

impl Button {
    const FIXED_SIZE: usize = 2 * mem::size_of::<usize>();

    pub fn new(component_id: u64, parent_component_id: u64, icon_name: &str, text: &str) -> Self {
        Button {
            base: crate::Component {
                component_id: component_id,
                parent_component_id: parent_component_id
            },
            icon_name: icon_name.to_string(),
            text: text.to_string()
        }
    }
}

impl ChannelObject for Button {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        core::ptr::copy(mem::transmute::<&Button, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamic size fields
        let icon_name_length = self.icon_name.len();
        *(pointer as *mut usize) = icon_name_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.icon_name.as_ptr(), pointer, icon_name_length);
        let pointer = pointer.offset(icon_name_length as isize);

        let text_length = self.text.len();
        *(pointer as *mut usize) = text_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.text.as_ptr(), pointer, text_length);
        // let pointer = pointer.offset(text_length as isize);

        // return used size
        Self::FIXED_SIZE + mem::size_of::<usize>() + icon_name_length + mem::size_of::<usize>() + text_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = Button::default();
        
        // read fixed size fields
        core::ptr::copy(pointer as *mut u8, mem::transmute::<&Button, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamic size fields
        let icon_name_length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.icon_name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, icon_name_length)).to_owned();
        let pointer = pointer.offset(icon_name_length as isize);

        let text_length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, text_length)).to_owned();
        // let pointer = pointer.offset(text_length as isize);

        object
    }
}
