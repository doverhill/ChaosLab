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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError};
use uuid::Uuid;
use crate::channel::StorageChannel;
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;

pub struct StorageServer {
    channels: BTreeMap<ChannelHandle, StorageChannel>,
    on_client_connected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_client_disconnected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_get_capabilities: Option<Box<dyn Fn(ChannelHandle)>>,
    on_list_objects: Option<Box<dyn Fn(ChannelHandle)>>,
    on_lock_object: Option<Box<dyn Fn(ChannelHandle)>>,
    on_unlock_object: Option<Box<dyn Fn(ChannelHandle)>>,
    on_read_object: Option<Box<dyn Fn(ChannelHandle)>>,
    on_write_object: Option<Box<dyn Fn(ChannelHandle)>>,
    on_watch_object: Option<Box<dyn Fn(ChannelHandle)>>,
    on_unwatch_object: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl StorageServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Self {
            channels: BTreeMap::new(),
            on_client_connected: None,
            on_client_disconnected: None,
            on_get_capabilities: None,
            on_list_objects: None,
            on_lock_object: None,
            on_unlock_object: None,
            on_read_object: None,
            on_write_object: None,
            on_watch_object: None,
            on_unwatch_object: None,
        })
    }

    pub fn on_client_connected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_connected = handler;
    }

    pub fn on_client_disconnected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_disconnected = handler;
    }

    pub fn watched_object_changed(&self, channel_handle: ChannelHandle, parameters: WatchedObjectChangedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::WatchedObjectChangedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn on_get_capabilities(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_get_capabilities = handler;
    }

    pub fn on_list_objects(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_list_objects = handler;
    }

    pub fn on_lock_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_lock_object = handler;
    }

    pub fn on_unlock_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_unlock_object = handler;
    }

    pub fn on_read_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_read_object = handler;
    }

    pub fn on_write_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_write_object = handler;
    }

    pub fn on_watch_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_watch_object = handler;
    }

    pub fn on_unwatch_object(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_unwatch_object = handler;
    }

}


