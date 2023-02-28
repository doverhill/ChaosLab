use crate::ChannelHandle;
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
    map_handle_0: HANDLE,
    pub map_pointer_0: *mut u8,
    map_handle_1: HANDLE,
    pub map_pointer_1: *mut u8,
}

impl Drop for Channel {
    fn drop(&mut self) {
        println!("dropping channel");
        if self.map_pointer_0 as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer_0 as *mut _) };
        }

        if self.map_handle_0 as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle_0) };
        }

        if self.map_pointer_1 as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer_1 as *mut _) };
        }

        if self.map_handle_1 as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle_1) };
        }
    }
}

impl Channel {
    pub fn new(handle: ChannelHandle, initial_size: usize) -> Self {
        // first buffer
        let memory_name = Self::get_map_name(handle, 0);
        let (map_handle_0, map_pointer_0) = Self::create_shared_memory(&memory_name, initial_size);
        map_handle_0.expect("Failed to create shared memory");

        // second buffer
        let memory_name = Self::get_map_name(handle, 1);
        let (map_handle_1, map_pointer_1) = Self::create_shared_memory(&memory_name, initial_size);
        map_handle_1.expect("Failed to create shared memory");

        Channel {
            map_handle_0: map_handle_0.unwrap(),
            map_pointer_0: map_pointer_0,
            map_handle_1: map_handle_1.unwrap(),
            map_pointer_1: map_pointer_1,
        }
    }

    // pub fn attach_observer(&mut self, observer: &'a mut CO) {
    //     self.observers.push(observer);
    // }

    // pub fn detach_observer(&mut self, observer: &'a mut CO) {
    //     if let Some(index) = self.observers.iter().position(|x| *x == observer) {
    //         self.observers.remove(index);
    //     }
    // }

    fn get_map_name(handle: ChannelHandle, id: usize) -> String {
        return format!("Local\\__chaos_channel_{}_{}", handle.raw_handle(), id);
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
