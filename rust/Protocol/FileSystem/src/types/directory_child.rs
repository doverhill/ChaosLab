use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const FILE_SYSTEM_DIRECTORY_CHILD_OBJECT_ID: usize = 1;

#[derive(Default)]
pub struct DirectoryChild {
    // fixed size fields
    pub is_directory: bool
    // dynamically sized fields
    pub name: String
}

impl DirectoryChild {
    const FIXED_SIZE: usize = mem::size_of::<bool>();

    pub fn new(name: &str, is_directory: bool) -> Self {
        DirectoryChild {
            name: name.to_string(),
            is_directory: is_directory
        }
    }
}

impl ChannelObject for DirectoryChild {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&DirectoryChild, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamically sized field name
        let length = self.name.len();
        *(pointer as *mut usize) = len;
        let pointer = pointer.offset(mem::size_of::<usize>());
        ptr::copy(self.name.as_ptr(), pointer, length);
    }

    unsafe fn from_channel(pointer: *mut u8) -> Self {
        let mut object = DirectoryChild::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&DirectoryChild, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamically sized field name
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>());
        object.name = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();
    }
}

