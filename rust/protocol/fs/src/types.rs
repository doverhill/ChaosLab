pub struct DirectoryChild {
    name: [u8; 100],
    is_directory: bool
}
pub struct FileInfo {
    name: [u8; 100],
    size: u64,
    created_at: u64
}
