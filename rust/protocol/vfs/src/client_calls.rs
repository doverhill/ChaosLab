extern crate chaos;
use chaos::channel::Channel;
use crate::types::*;
use crate::ipc::*;

#[allow(dead_code)]
pub fn vfs_mount(channel: Channel, path: Path) {
}

#[allow(dead_code)]
unsafe fn vfs_mount_raw(channel: Channel, path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn vfs_directory_list(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn vfs_directory_list_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn vfs_file_info(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn vfs_file_info_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn vfs_file_read(channel: Channel, full_path: Path) {
}

#[allow(dead_code)]
unsafe fn vfs_file_read_raw(channel: Channel, full_path: Path) -> ptr {
}

#[allow(dead_code)]
pub fn vfs_file_copy(channel: Channel, source_full_path: Path, target_full_path: Path) {
}

#[allow(dead_code)]
unsafe fn vfs_file_copy_raw(channel: Channel, source_full_path: Path, target_full_path: Path) -> ptr {
}

