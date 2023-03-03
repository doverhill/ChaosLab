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
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{StorageChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum StorageServerRequest<'a> {
    GetCapabilities,
    ListObjects(&'a ListObjectsParameters),
    LockObject(&'a LockObjectParameters),
    UnlockObject(&'a UnlockObjectParameters),
    ReadObject(&'a ReadObjectParameters),
    WriteObject(&'a WriteObjectParameters),
    WatchObject(&'a WatchObjectParameters),
    UnwatchObject(&'a UnwatchObjectParameters),
}

pub trait StorageServerObserver {
    fn handle_storage_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_storage_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, call_id: u64, request: StorageServerRequest);
}

pub struct StorageServer {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, StorageChannel>,
}

impl StorageServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Rc<RefCell<Self>>, StormError> {
        let service_handle = process.create_service("storage", vendor_name, device_name, device_id)?;
        Ok(Rc::new(RefCell::new(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })))
    }

    pub fn process_event(&mut self, process: &mut StormProcess, event: &StormEvent, observer: &mut impl StorageServerObserver) {
        match event {
            StormEvent::ServiceConnected(service_handle, channel_handle) => {
                if *service_handle == self.service_handle {
                    println!("StorageServer: client connected");
                    process.initialize_channel(*channel_handle, 4096);
                    let channel = StorageChannel::new(process.get_channel_address(*channel_handle, 0).unwrap(), process.get_channel_address(*channel_handle, 1).unwrap(), true);
                    self.channels.insert(*channel_handle, channel);
                    observer.handle_storage_client_connected(*service_handle, *channel_handle);
                }
            }
            StormEvent::ChannelSignalled(channel_handle) => {
                if let Some(channel) = self.channels.get(&channel_handle) {
                    while let Some(message) = channel.find_message() {
                        unsafe {
                            match (*message).message_id {
                                GET_CAPABILITIES_PARAMETERS =>  {
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, StorageServerRequest::GetCapabilities);
                                    channel.unlink_message(message, false);
                                }
                                LIST_OBJECTS_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    ListObjectsParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const ListObjectsParameters;
                                    let request = StorageServerRequest::ListObjects(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                LOCK_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    LockObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const LockObjectParameters;
                                    let request = StorageServerRequest::LockObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                UNLOCK_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    UnlockObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const UnlockObjectParameters;
                                    let request = StorageServerRequest::UnlockObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                READ_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    ReadObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const ReadObjectParameters;
                                    let request = StorageServerRequest::ReadObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                WRITE_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    WriteObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const WriteObjectParameters;
                                    let request = StorageServerRequest::WriteObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                WATCH_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    WatchObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const WatchObjectParameters;
                                    let request = StorageServerRequest::WatchObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                UNWATCH_OBJECT_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    UnwatchObjectParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const UnwatchObjectParameters;
                                    let request = StorageServerRequest::UnwatchObject(parameters.as_ref().unwrap());
                                    observer.handle_storage_request(self.service_handle, *channel_handle, (*message).call_id, request);
                                    channel.unlink_message(message, false);
                                }
                                _ => {}
                            }
                        }
                    }
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

    pub fn watched_object_changed(&mut self, channel_handle: ChannelHandle, parameters: &WatchedObjectChangedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(WATCHED_OBJECT_CHANGED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn get_capabilities_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &GetCapabilitiesReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(GET_CAPABILITIES_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }
    pub fn list_objects_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &ListObjectsReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(LIST_OBJECTS_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }
    pub fn lock_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &LockObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(LOCK_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }
    pub fn read_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &ReadObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(READ_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }
    pub fn watch_object_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &WatchObjectReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(WATCH_OBJECT_RETURNS, false);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }
}


