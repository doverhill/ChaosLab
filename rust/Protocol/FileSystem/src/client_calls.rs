extern crate chaos;
use chaos::channel::Channel;
use crate::types::*;
use crate::ipc::*;

#[allow(dead_code)]
pub fn FileSystem_directory_list(channel: Channel, full_path: &str) {
}

#[allow(dead_code)]
unsafe fn FileSystem_directory_list_raw(channel: Channel, full_path: &str) -> ptr {
}

#[allow(dead_code)]
pub fn FileSystem_file_info(channel: Channel, full_path: &str) {
}

#[allow(dead_code)]
unsafe fn FileSystem_file_info_raw(channel: Channel, full_path: &str) -> ptr {
}

#[allow(dead_code)]
pub fn FileSystem_file_read(channel: Channel, full_path: &str) {
}

#[allow(dead_code)]
unsafe fn FileSystem_file_read_raw(channel: Channel, full_path: &str) -> ptr {
}

#[allow(dead_code)]
pub fn FileSystem_file_copy(channel: Channel, source_full_path: &str, target_full_path: &str) {
}

#[allow(dead_code)]
unsafe fn FileSystem_file_copy_raw(channel: Channel, source_full_path: &str, target_full_path: &str) -> ptr {
}

