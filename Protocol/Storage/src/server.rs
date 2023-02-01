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
use alloc::vec::Vec;

pub enum StorageServerRequest {
    GetCapabilities,
    ListObjects(ListObjectsParameters),
    LockObject(LockObjectParameters),
    UnlockObject(UnlockObjectParameters),
    ReadObject(ReadObjectParameters),
    WriteObject(WriteObjectParameters),
    WatchObject(WatchObjectParameters),
    UnwatchObject(UnwatchObjectParameters),
}

pub trait StorageServerObserver {
    fn handle_storage_client_connected(service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_client_disconnected(service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_request(service_handle: ServiceHandle, channel_handle: ChannelHandle, request: StorageServerRequest);
}

pub struct StorageServer<'a, T: StorageServerObserver + PartialEq> {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, StorageChannel>,
    observers: Vec<&'a T>,
}

impl<'a, T: StorageServerObserver + PartialEq> StorageServer<'a, T> {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
            observers: Vec::new(),
        })
    }

    pub fn attach_observer(&mut self, observer: &'a T) {
        self.observers.push(observer);
    }

    pub fn detach_observer(&mut self, observer: &'a T) {
        if let Some(index) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(index);
        }
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


