extern crate chaos;
use chaos::channel::Channel;
use crate::types::*;
use crate::ipc::*;

#[allow(dead_code)]
pub fn fs_directory_list(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn fs_directory_list_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn fs_file_info(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn fs_file_info_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn fs_file_read(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn fs_file_read_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn fs_file_copy(channel: Channel, source_full_path: Path, target_full_path: Path) {
}

#[allow(dead_code)]
unsafe fn fs_file_copy_raw(channel: Channel, source_full_path: Path, target_full_path: Path) -> ptr {
}

