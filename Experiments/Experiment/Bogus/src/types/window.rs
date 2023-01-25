use library_chaos::ChannelObject;
use std::mem;

pub const BOGUS_TYPE_WINDOW_OBJECT_ID: usize = 7;
#[derive(Default)]
pub struct Window {
    pub base: crate::Component,
    pub title: String    
}

impl Window {
    const FIXED_SIZE: usize = 2 * mem::size_of::<usize>();

    pub fn new(component_id: u64, parent_component_id: u64, title: &str) -> Self {
        Window {
            base: crate::Component {
                component_id: component_id,
                parent_component_id: parent_component_id
            },
            title: title.to_string()
        }
    }
}

impl ChannelObject for Window {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        core::ptr::copy(mem::transmute::<&Window, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamic size fields
        let title_length = self.title.len();
        *(pointer as *mut usize) = title_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.title.as_ptr(), pointer, title_length);

        // return used size
        Self::FIXED_SIZE + mem::size_of::<usize>() + title_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = Window::default();
        
        // read fixed size fields
        core::ptr::copy(pointer as *mut u8, mem::transmute::<&Window, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamic size fields
        let title_length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.title = std::str::from_utf8_unchecked(std::slice::from_raw_parts(pointer as *const u8, title_length)).to_owned();

        object
    }
}
