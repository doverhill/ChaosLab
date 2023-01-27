#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

use alloc::boxed::Box;
use library_chaos::{StormProcess, StormHandle, Service};
use uuid::Uuid;

pub struct StorageClient {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_watched_object_changed: Option<Box<dyn FnMut()>>,
}

impl StorageClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        match syscalls::connect("BogusAuto", None, None, None, 4096) {
            Ok(channel_reference) => {
                Ok(Self::from_channel(channel_reference, implementation))
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to connect to BogusAuto service").unwrap();
                Err(error)
            }
        }
    }

    pub fn get_capabilities() {
    }

    pub fn list_objects() {
    }

    pub fn lock_object() {
    }

    pub fn unlock_object() {
    }

    pub fn read_object() {
    }

    pub fn write_object() {
    }

    pub fn watch_object() {
    }

    pub fn unwatch_object() {
    }

    pub fn on_watched_object_changed() {
    }

}


