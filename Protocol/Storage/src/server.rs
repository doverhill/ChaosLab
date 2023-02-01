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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, ServiceObserver, ChannelObserver};
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

pub struct StorageServer<'a, T: StorageServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, StorageChannel>,
    observers: Vec<&'a T>,
    so: Option<&'a SO>,
    co: Option<&'a CO>,
}

impl<'a, T: StorageServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> StorageServer<'a, T, SO, CO> {
    pub fn create(process: &mut StormProcess<SO, CO>, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
            observers: Vec::new(),
            so: None,
            co: None,
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
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::WatchedObjectChangedParameters as u64);
            }
        }
    }

}


