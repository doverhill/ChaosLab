use library_chaos::ChannelObject;
use std::mem;

pub const BOGUS_TYPE_FILE_INFO_OBJECT_ID: usize = 5;
#[derive(Default)]
pub struct FileInfo {
    // fixed fields
    pub size: usize,

    // dynamix size fields
    pub path: String
}

impl FileInfo {
    const FIXED_SIZE: usize = mem::size_of::<usize>();

    pub fn new(path: &str, size: usize) -> FileInfo {
        FileInfo {
            path: path.to_string(),
            size: size
        }
    }
}

impl ChannelObject for FileInfo {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields
        core::ptr::copy(mem::transmute::<&FileInfo, *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // write dynamic size fields
        // PATH
        let path_length = self.path.len();
        *(pointer as *mut usize) = path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.path.as_ptr(), pointer, path_length);

        // return used size
        Self::FIXED_SIZE + mem::size_of::<usize>() + path_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = FileInfo::default();
        
        // read fixed size fields
        core::ptr::copy(pointer as *mut u8, mem::transmute::<&FileInfo, *mut u8>(&object), Self::FIXED_SIZE);
        let pointer = pointer.offset(Self::FIXED_SIZE as isize);

        // read dynamic size fields
        let path_length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.path = std::str::from_utf8_unchecked(std::slice::from_raw_parts(pointer as *const u8, path_length)).to_owned();

        object
    }
}
