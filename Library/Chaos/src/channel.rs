use crate::{ syscalls, StormHandle, StormProcess, StormError, StormAction };
use std::fmt;
use ::winapi::{
    shared::ntdef::NULL,
    um::{
        handleapi::{ CloseHandle, INVALID_HANDLE_VALUE },
        memoryapi::{ CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS },
        winnt::{ HANDLE, PAGE_READWRITE }
    }
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::mem;
use std::slice;

pub struct StormChannel {
    pub handle: StormHandle,
    map_handle: HANDLE,
    map_pointer: *mut u8,
    map_capacity: usize,
}

impl Drop for StormChannel {
    fn drop(&mut self) {
        println!("dropping channel");
        if self.map_pointer as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer as *mut _) };
        }

        if self.map_handle as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle) };
        }

        syscalls::channel_destroy(self.handle).unwrap();
    }
}

impl StormChannel {
    pub fn new(handle: Handle, size: usize) -> Self {
        let memory_name = &Channel::get_map_name(&handle);
        let (map_handle, map_pointer) = Channel::create_shared_memory(memory_name, size);
        map_handle.expect("Failed to create shared memory");

        Self {
            handle: handle,
            map_handle: map_handle.unwrap(),
            map_pointer: map_pointer,
            map_capacity: size,
            body_pointer: map_pointer,
            allocation_pointer: map_pointer,
            message_handler: None
        }
    }

    fn create_shared_memory(name: &str, size: usize) -> (Option<HANDLE>, *mut u8) {
        let high_size: u32 = ((size & 0xFFFF_FFFF_0000_0000_usize) >> 32) as u32;
        let low_size: u32 = (size & 0xFFFF_FFFF_usize) as u32;
        let unique_id: Vec<u16> = OsStr::new(name).encode_wide().chain(once(0)).collect();

        let map_handle = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                null_mut(),
                PAGE_READWRITE,
                high_size,
                low_size,
                unique_id.as_ptr(),
            )
        };

        if map_handle == NULL {
            return (None, null_mut());
        }

        let map_pointer = unsafe { MapViewOfFile(map_handle, FILE_MAP_ALL_ACCESS, 0, 0, 0) } as _;

        (Some(map_handle), map_pointer)
    }

    fn get_map_name(handle: &Handle) -> String {
        return format!("Local\\__chaos_channel_{}", handle);
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[CHANNEL: handle={}, map_handle={:?}, buffer={:p}]", self.handle, self.map_handle, self.map_pointer)
    }
}
