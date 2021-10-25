use crate::handle::Handle;
use std::fmt;
use std::sync::Mutex;
use std::collections::HashMap;
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

pub struct Channel {
    channel_handle: Handle,
    map_handle: HANDLE,
    pub map_pointer: *mut u8
}

impl Drop for Channel {
    fn drop(&mut self) {
        if self.map_pointer as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer as *mut _) };
        }

        if self.map_handle as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle) };
        }
    }
}

impl Channel {
    pub fn new(channel_handle: Handle) -> Channel {
        let memory_name = &Channel::get_map_name(&channel_handle);
        let map_size: u64 = 4096;

        let (map_handle, map_pointer) = Channel::create_shared_memory(memory_name, map_size);
        
        map_handle.expect("Failed to create shared memory");

        Channel {
            channel_handle: channel_handle,
            map_handle: map_handle.unwrap(),
            map_pointer: map_pointer
        }
    }

    fn create_shared_memory(name: &str, size: u64) -> (Option<HANDLE>, *mut u8) {
        let high_size: u32 = ((size & 0xFFFF_FFFF_0000_0000_u64) >> 32) as u32;
        let low_size: u32 = (size & 0xFFFF_FFFF_u64) as u32;
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

    fn get_map_name(channel_handle: &Handle) -> String {
        return format!("Local\\__chaos_channel_{}", channel_handle.id);
    }

    pub fn write(&self, value: u8) -> () {
        unsafe {
            println!("reading");
            let v = std::ptr::read_volatile(self.map_pointer);
            println!("read {}", v);
            std::ptr::write_volatile(self.map_pointer, value);
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[CHANNEL: handle={}, map_handle={:?}, buffer={:p}]", self.channel_handle, self.map_handle, self.map_pointer)
    }
}

lazy_static! {
    static ref ON_MESSAGE: Mutex<HashMap<u64, fn(Channel) -> ()>> = {
        Mutex::new(HashMap::new())
    };
}

pub fn on_message(channel: Channel, handler: Option<fn(Channel) -> ()>) {
    match handler {
        Some(f) => {
            ON_MESSAGE.lock().unwrap().insert(channel.channel_handle.id, f);
        },
        None => {
            ON_MESSAGE.lock().unwrap().remove(&channel.channel_handle.id);
        }
    }
}
