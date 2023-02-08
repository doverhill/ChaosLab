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

pub enum StorageClientEvent {
    WatchedObjectChanged(WatchedObjectChangedParameters),
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
        let channel = StorageChannel::new(process.get_channel_address(channel_handle).unwrap(), false);
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl StorageClientObserver) {
        match event {
            StormEvent::ChannelSignalled(channel_handle) => {
                if *channel_handle == self.channel_handle {
                    println!("StorageClient: got event");
                    // observer.handle_storage_event(*channel_handle, event);
                }
            }
            _ => {}
        }
    }

    pub fn get_capabilities(&self, process: &StormProcess) -> Result<FromChannel<&GetCapabilitiesReturns>, StormError> {
        let message = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, false);
        self.channel.commit_message(0);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(GET_CAPABILITIES_RETURNS) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetCapabilitiesReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut GetCapabilitiesReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn list_objects(&self, process: &StormProcess, parameters: &ListObjectsParameters) -> Result<FromChannel<&ListObjectsReturns>, StormError> {
        let message = self.channel.prepare_message(LIST_OBJECTS_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(LIST_OBJECTS_RETURNS) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ListObjectsReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut ListObjectsReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn lock_object(&self, process: &StormProcess, parameters: &LockObjectParameters) -> Result<FromChannel<&LockObjectReturns>, StormError> {
        let message = self.channel.prepare_message(LOCK_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(LOCK_OBJECT_RETURNS) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { LockObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut LockObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unlock_object(&self, parameters: &UnlockObjectParameters) {
        let message = self.channel.prepare_message(UNLOCK_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn read_object(&self, process: &StormProcess, parameters: &ReadObjectParameters) -> Result<FromChannel<&ReadObjectReturns>, StormError> {
        let message = self.channel.prepare_message(READ_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(READ_OBJECT_RETURNS) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ReadObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut ReadObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn write_object(&self, parameters: &WriteObjectParameters) {
        let message = self.channel.prepare_message(WRITE_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn watch_object(&self, process: &StormProcess, parameters: &WatchObjectParameters) -> Result<FromChannel<&WatchObjectReturns>, StormError> {
        let message = self.channel.prepare_message(WATCH_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(WATCH_OBJECT_RETURNS) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { WatchObjectReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut WatchObjectReturns;
            Ok(FromChannel::new(unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unwatch_object(&self, parameters: &UnwatchObjectParameters) {
        let message = self.channel.prepare_message(UNWATCH_OBJECT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

}


