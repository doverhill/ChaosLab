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
use crate::channel::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum StorageServerRequest {
    GetCapabilities,
    ListObjects(FromChannel<ListObjectsParameters>),
    LockObject(FromChannel<LockObjectParameters>),
    UnlockObject(FromChannel<UnlockObjectParameters>),
    ReadObject(FromChannel<ReadObjectParameters>),
    WriteObject(FromChannel<WriteObjectParameters>),
    WatchObject(FromChannel<WatchObjectParameters>),
    UnwatchObject(FromChannel<UnwatchObjectParameters>),
}

pub enum StorageServerChannelEvent {
    ClientConnected(ServiceHandle, ChannelHandle),
    ClientDisconnected(ServiceHandle, ChannelHandle),
    ClientRequest(ServiceHandle, ChannelHandle, u64, StorageServerRequest),
}

pub struct StorageServer {
    current_event: Option<StormEvent>,
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, StorageChannel>,
}

impl StorageServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Self {
            current_event: None,
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &mut StormProcess) -> Option<StorageServerChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ServiceConnected(service_handle, channel_handle) => {
                    self.current_event = None;
                    if service_handle == self.service_handle {
                        println!("StorageServer: client connected");
                        process.initialize_channel(channel_handle, 1048576);
                        let channel = StorageChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), true);
                        self.channels.insert(channel_handle, channel);
                        Some(StorageServerChannelEvent::ClientConnected(service_handle, channel_handle))
                    }
                    else {
                        None
                    }
                }
                StormEvent::ChannelSignalled(channel_handle) => {
                    if let Some(channel) = self.channels.get(&channel_handle) {
                        if let Some(message) = channel.find_message() {
                            unsafe {
                                match (*message).message_id {
                                    GET_CAPABILITIES_PARAMETERS => {
                                        channel.unlink_message(message, false);
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, StorageServerRequest::GetCapabilities))
                                    },
                                    LIST_OBJECTS_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        ListObjectsParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::ListObjects(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    LOCK_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        LockObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::LockObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    UNLOCK_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        UnlockObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::UnlockObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    READ_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        ReadObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::ReadObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    WRITE_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WriteObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::WriteObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    WATCH_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WatchObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::WatchObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    UNWATCH_OBJECT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        UnwatchObjectParameters::reconstruct_at_inline(address);
                                        let request = StorageServerRequest::UnwatchObject(FromChannel::new(channel.rx_channel_address, message));
                                        Some(StorageServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    _ => { panic!("StorageServer: Unknown message received"); }
                                }
                            }
                        }
                        else {
                            self.current_event = None;
                            None
                        }
                    }
                    else {
                        self.current_event = None;
                        None
                    }
                }
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if let Some(_) = self.channels.get(&channel_handle) {
                        println!("StorageServer: client disconnected");
                        Some(StorageServerChannelEvent::ClientDisconnected(self.service_handle, channel_handle))
                    }
                    else {
                        None
                    }
                }
            }
        }
        else {
            None
        }
    }

    pub fn watched_object_changed(&mut self, channel_handle: ChannelHandle, parameters: &WatchedObjectChangedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(WATCHED_OBJECT_CHANGED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn get_capabilities_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &GetCapabilitiesReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(GET_CAPABILITIES_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
    pub fn list_objects_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &ListObjectsReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(LIST_OBJECTS_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
    pub fn lock_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &LockObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(LOCK_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
    pub fn read_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &ReadObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(READ_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
    pub fn watch_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &WatchObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(WATCH_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
}


