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
use crate::channel::{StorageChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;

pub struct StorageServer<'a> {
    channels: BTreeMap<ChannelHandle, StorageChannel>,
    on_client_connected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_client_disconnected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_get_capabilities: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_list_objects: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_lock_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_unlock_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_read_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_write_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_watch_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
    on_unwatch_object: Option<Box<dyn Fn(ChannelHandle) + 'a>>,
}

impl<'a> StorageServer<'a> {
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

    pub fn on_client_connected(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_client_connected = Some(Box::new(handler));
    }

    pub fn clear_on_client_connected(&mut self) {
        self.on_client_connected = None;
    }

    pub fn on_client_disconnected(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_client_disconnected = Some(Box::new(handler));
    }

    pub fn clear_on_client_disconnected(&mut self) {
        self.on_client_disconnected = None;
    }

    pub fn watched_object_changed(&self, channel_handle: ChannelHandle, parameters: WatchedObjectChangedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::WatchedObjectChangedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::send_channel_message(channel_handle, MessageIds::WatchedObjectChangedParameters as u64);
            }
        }
    }

    pub fn on_get_capabilities(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_get_capabilities = Some(Box::new(handler));
    }

    pub fn clear_on_get_capabilities(&mut self) {
        self.on_get_capabilities = None;
    }

    pub fn on_list_objects(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_list_objects = Some(Box::new(handler));
    }

    pub fn clear_on_list_objects(&mut self) {
        self.on_list_objects = None;
    }

    pub fn on_lock_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_lock_object = Some(Box::new(handler));
    }

    pub fn clear_on_lock_object(&mut self) {
        self.on_lock_object = None;
    }

    pub fn on_unlock_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_unlock_object = Some(Box::new(handler));
    }

    pub fn clear_on_unlock_object(&mut self) {
        self.on_unlock_object = None;
    }

    pub fn on_read_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_read_object = Some(Box::new(handler));
    }

    pub fn clear_on_read_object(&mut self) {
        self.on_read_object = None;
    }

    pub fn on_write_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_write_object = Some(Box::new(handler));
    }

    pub fn clear_on_write_object(&mut self) {
        self.on_write_object = None;
    }

    pub fn on_watch_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_watch_object = Some(Box::new(handler));
    }

    pub fn clear_on_watch_object(&mut self) {
        self.on_watch_object = None;
    }

    pub fn on_unwatch_object(&mut self, handler: impl Fn(ChannelHandle) + 'a) {
        self.on_unwatch_object = Some(Box::new(handler));
    }

    pub fn clear_on_unwatch_object(&mut self) {
        self.on_unwatch_object = None;
    }

}


