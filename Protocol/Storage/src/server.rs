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
use library_chaos::{StormProcess, StormHandle};
use uuid::Uuid;

pub struct StorageServer {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_get_capabilities: Option<Box<dyn FnMut()>>,
    on_list_objects: Option<Box<dyn FnMut()>>,
    on_lock_object: Option<Box<dyn FnMut()>>,
    on_unlock_object: Option<Box<dyn FnMut()>>,
    on_read_object: Option<Box<dyn FnMut()>>,
    on_write_object: Option<Box<dyn FnMut()>>,
    on_watch_object: Option<Box<dyn FnMut()>>,
    on_unwatch_object: Option<Box<dyn FnMut()>>,
}

impl StorageServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<Self> {
    }

    pub fn watched_object_changed() {
    }

    pub fn on_get_capabilities() {
    }

    pub fn on_list_objects() {
    }

    pub fn on_lock_object() {
    }

    pub fn on_unlock_object() {
    }

    pub fn on_read_object() {
    }

    pub fn on_write_object() {
    }

    pub fn on_watch_object() {
    }

    pub fn on_unwatch_object() {
    }

}


