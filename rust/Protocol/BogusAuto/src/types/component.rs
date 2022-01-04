use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const BOGUS_AUTO_COMPONENT_OBJECT_ID: usize = 2;

#[derive(Default)]
pub struct Component {
    // fixed size fields
    pub component_id: u64,
    pub parent_component_id: u64
    // dynamically sized fields
}

impl Component {
    const FIXED_SIZE: usize = mem::size_of::<u64>() + mem::size_of::<u64>();

    pub fn new(component_id: u64, parent_component_id: u64) -> Self {
        Component {
            component_id: component_id,
            parent_component_id: parent_component_id
        }
    }
}

impl ChannelObject for Component {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&Component, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = Component::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&Component, *mut u8>(&object), Self::FIXED_SIZE);
    }
}

