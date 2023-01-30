use crate::{ syscalls, ChannelHandle };
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


pub struct Channel {
    map_handle: HANDLE,
    pub map_pointer: *mut u8,
    pub on_messaged: Option<Box<dyn Fn(ChannelHandle, u64)>>,
    pub on_destroyed: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl Channel {
    pub fn new(handle: ChannelHandle) -> Self {
        let memory_name = Self::get_map_name(handle);
        let (map_handle, map_pointer) = Self::create_shared_memory(&memory_name, 1024 * 1024);
        map_handle.expect("Failed to create shared memory");

        Channel {
            map_handle: map_handle.unwrap(),
            map_pointer: map_pointer,
            on_messaged: None,
            on_destroyed: None,
        }
    }

    fn get_map_name(handle: ChannelHandle) -> String {
        return format!("Local\\__chaos_channel_{}", handle.raw_handle());
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
}

impl Drop for Channel {
    fn drop(&mut self) {
        println!("dropping channel");
        if self.map_pointer as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer as *mut _) };
        }

        if self.map_handle as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle) };
        }
    }
}
