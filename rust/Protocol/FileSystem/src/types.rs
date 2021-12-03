#[allow(dead_code)]
pub struct Path {
    path: [u8; 1024]
}

#[allow(dead_code)]
impl Path {
    pub fn new(path: &str) -> Path {
        let constructed_path = Path {
            path: [0u8; 1024]
        };
        unsafe { core::ptr::copy(path.as_ptr(), core::ptr::addr_of!(constructed_path.path) as *mut u8, core::cmp::min(1023, path.len())); }
        constructed_path
    }

    pub fn get_path(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.path) }
    }
}

#[allow(dead_code)]
pub struct DirectoryChild {
    name: [u8; 256],
    is_directory: bool
}

#[allow(dead_code)]
impl DirectoryChild {
    pub fn new(name: &str, is_directory: bool) -> DirectoryChild {
        let constructed_directory_child = DirectoryChild {
            name: [0u8; 256],
            is_directory: is_directory
        };
        unsafe { core::ptr::copy(name.as_ptr(), core::ptr::addr_of!(constructed_directory_child.name) as *mut u8, core::cmp::min(255, name.len())); }
        constructed_directory_child
    }

    pub fn get_name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name) }
    }
}

#[allow(dead_code)]
pub struct FileInfo {
    name: [u8; 256],
    size: u64,
    created_at: i32
}

#[allow(dead_code)]
impl FileInfo {
    pub fn new(name: &str, size: u64, created_at: i32) -> FileInfo {
        let constructed_file_info = FileInfo {
            name: [0u8; 256],
            size: size,
            created_at: created_at
        };
        unsafe { core::ptr::copy(name.as_ptr(), core::ptr::addr_of!(constructed_file_info.name) as *mut u8, core::cmp::min(255, name.len())); }
        constructed_file_info
    }

    pub fn get_name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name) }
    }
}

