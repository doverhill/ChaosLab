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
use crate::channel::{StorageChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::vec::Vec;

pub enum StorageClientEvent<'a> {
    WatchedObjectChanged(&'a WatchedObjectChangedParameters),
}

pub trait StorageClientObserver {
    fn handle_storage_event(&mut self, channel_handle: ChannelHandle, event: StorageClientEvent);
}

pub struct StorageClient {
    channel_handle: ChannelHandle,
    channel: StorageChannel,
}

impl StorageClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("storage", None, None, None, 4096)?;
        let channel = StorageChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl StorageClientObserver) {
        match event {
            StormEvent::ChannelSignalled(channel_handle) => {
                if *channel_handle == self.channel_handle {
                    while let Some(message) = self.channel.find_message() {
                        unsafe {
                            match (*message).message_id {
                                WATCHED_OBJECT_CHANGED_PARAMETERS =>  {
                                    println!("got WATCHED_OBJECT_CHANGED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    WatchedObjectChangedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const WatchedObjectChangedParameters;
                                    let request = StorageClientEvent::WatchedObjectChanged(parameters.as_ref().unwrap());
                                    observer.handle_storage_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                _ => {}
                            }
                        }
                    }
                    // observer.handle_storage_event(*channel_handle, event);
                }
            }
            _ => {}
        }
    }

    pub fn get_capabilities(&mut self, process: &StormProcess) -> Result<FromChannel<&GetCapabilitiesReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, false);
        self.channel.commit_message(0);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetCapabilitiesReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut GetCapabilitiesReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn list_objects(&mut self, process: &StormProcess, parameters: &ListObjectsParameters) -> Result<FromChannel<&ListObjectsReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(LIST_OBJECTS_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ListObjectsReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut ListObjectsReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn lock_object(&mut self, process: &StormProcess, parameters: &LockObjectParameters) -> Result<FromChannel<&LockObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(LOCK_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { LockObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut LockObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unlock_object(&mut self, parameters: &UnlockObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(UNLOCK_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn read_object(&mut self, process: &StormProcess, parameters: &ReadObjectParameters) -> Result<FromChannel<&ReadObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(READ_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ReadObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut ReadObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn write_object(&mut self, parameters: &WriteObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn watch_object(&mut self, process: &StormProcess, parameters: &WatchObjectParameters) -> Result<FromChannel<&WatchObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(WATCH_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { WatchObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut WatchObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unwatch_object(&mut self, parameters: &UnwatchObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(UNWATCH_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

}


