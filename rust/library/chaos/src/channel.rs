use crate::{ syscalls, handle::Handle, process::Process, error::Error, action::Action };
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

lazy_static! {
    static ref CHANNELS: Mutex<HashMap<Handle, Arc<Mutex<Channel>>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct Channel {
    handle: Handle,
    map_handle: HANDLE,
    map_pointer: *mut u8,
    on_message: Option<fn(&Arc<Mutex<Channel>>, u64) -> ()>
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}

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

impl Channel {
    pub fn new(handle: Handle, size: usize) -> Arc<Mutex<Channel>> {
        let memory_name = &Channel::get_map_name(&handle);
        let (map_handle, map_pointer) = Channel::create_shared_memory(memory_name, size);
        map_handle.expect("Failed to create shared memory");

        let channel = Arc::new(Mutex::new(Channel {
            handle: handle,
            map_handle: map_handle.unwrap(),
            map_pointer: map_pointer,
            on_message: None
        }));

        let mut channels = CHANNELS.lock().unwrap();
        channels.insert(handle, channel.clone());

        channel
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

    pub fn on_message(&mut self, handler: fn(&Arc<Mutex<Channel>>, u64) -> ()) -> Option<Error> {
        match self.on_message {
            Some(_) => {
                Some(Error::AlreadyExists)
            },
            None => {
                self.on_message = Some(handler);
                None
            }
        }
    }

    pub(crate) fn messaged(handle: Handle, message: u64) {
        Process::emit_debug(&format!("Channel {} got message {}", handle, message));

        let channels = CHANNELS.lock().unwrap();
        if let Some(channel_wrap) = channels.get(&handle) {
            let channel = channel_wrap.lock().unwrap();
            if let Some(handler) = channel.on_message {
                drop(channel); // release mutex
                handler(channel_wrap, message);
            }
        }
    }

    pub fn set<T: Copy>(&self, data: T) {
        unsafe {
            *(self.map_pointer as *mut T) = data;
        }
    }

    pub fn get<T>(&self) -> &T {
        unsafe {
            &*(self.map_pointer as *const T)
        }
    }

    pub fn send(&self, message: u64) {
        syscalls::channel_message(self.handle, message);
    }

    pub fn call_sync(&self, message: u64, reply: u64, timeout_milliseconds: i32) -> Option<Error> {
        syscalls::channel_message(self.handle, message);
        match syscalls::event_wait(-1) {
            Ok((target_handle, argument_handle, action, parameter)) => {
                if target_handle != self.handle {
                    Some(Error::General)
                }
                else if action != Action::ChannelMessaged {
                    Some(Error::General)
                }
                else if parameter != reply {
                    Some(Error::General)
                }
                else {
                    None
                }
            },
            Err(error) => {
                Some(error)
            }
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[CHANNEL: handle={}, map_handle={:?}, buffer={:p}]", self.handle, self.map_handle, self.map_pointer)
    }
}
