use library_chaos::ChannelObject;
use core::{ mem, ptr, str, slice };

pub const BOGUS_AUTO_FILE_INFO_OBJECT_ID: usize = 1;

#[derive(Default)]
pub struct FileInfo {
    // fixed size fields
    pub size: usize,
    // dynamically sized fields
    pub path: String
}

impl FileInfo {
    const FIXED_SIZE: usize = mem::size_of::<usize>();

    pub fn new(path: &str, size: usize) -> Self {
        FileInfo {
            path: path.to_string(),
            size: size
        }
    }
}

impl ChannelObject for FileInfo {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        ptr::copy(mem::transmute::<&FileInfo, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamically sized field path
        let path_length = self.path.len();
        *(pointer as *mut usize) = path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        ptr::copy(self.path.as_ptr(), pointer, path_length);

        Self::FIXED_SIZE + mem::size_of::<usize>() + path_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = FileInfo::default();

        // read fixed size fields
        ptr::copy(pointer as *mut u8, mem::transmute::<&FileInfo, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamically sized field path
        let length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.path = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();

        object
    }
}

