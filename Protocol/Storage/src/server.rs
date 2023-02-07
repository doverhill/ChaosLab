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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{StorageChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
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
    fn handle_storage_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: StorageServerRequest);
}

pub struct StorageServer {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, StorageChannel>,
}

impl StorageServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn process_event(&mut self, process: &mut StormProcess, event: &StormEvent, observer: &mut impl StorageServerObserver) {
        match event {
            StormEvent::ServiceConnected(service_handle, channel_handle) => {
                println!("{:?} == {:?}?", *service_handle, self.service_handle);
                if *service_handle == self.service_handle {
                    println!("StorageServer: client connected");
                    process.initialize_channel(*channel_handle, 4096);
                    let channel = StorageChannel::new(process.get_channel_address(*channel_handle).unwrap(), true);
                    self.channels.insert(*channel_handle, channel);
                    observer.handle_storage_client_connected(*service_handle, *channel_handle);
                }
            }
            StormEvent::ChannelSignalled(channel_handle) => {
                if let Some(channel) = self.channels.get(&channel_handle) {
                    println!("StorageServer: client request");
                    while let Some(message) = channel.find_message() {
                    }
                    // observer.handle_storage_request(self.service_handle, channel_handle, request);
                }
            }
            StormEvent::ChannelDestroyed(channel_handle) => {
                if let Some(_) = self.channels.get(&channel_handle) {
                    println!("StorageServer: client disconnected");
                    observer.handle_storage_client_disconnected(self.service_handle, *channel_handle);
                }
            }
        }
    }

    pub fn watched_object_changed(&self, channel_handle: ChannelHandle, parameters: WatchedObjectChangedParameters) {
        println!("StorageServer::watched_object_changed");
        if let Some(channel) = self.channels.get(&channel_handle) {
            println!("found channel");
            let message = channel.prepare_message(WATCHED_OBJECT_CHANGED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

}


