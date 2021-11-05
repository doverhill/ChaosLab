pub fn directory_list(full_path: [u8; 100]) -> DirectoryChild {
}

unsafe pub fn directory_list_raw(full_path: [u8; 100]) -> DirectoryChild {
}
pub fn file_info(full_path: [u8; 100]) -> FileInfo {
}

unsafe pub fn file_info_raw(full_path: [u8; 100]) -> FileInfo {
}
pub fn file_read(full_path: [u8; 100]) -> u8 {
}

unsafe pub fn file_read_raw(full_path: [u8; 100]) -> u8 {
}
pub fn file_copy(source_full_path: [u8; 100], target_full_path: [u8; 100]) {
}

unsafe pub fn file_copy_raw(source_full_path: [u8; 100], target_full_path: [u8; 100]) {
}
