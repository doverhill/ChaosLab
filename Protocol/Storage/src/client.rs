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
use crate::channel::{StorageChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::vec::Vec;

pub enum StorageClientEvent {
    WatchedObjectChanged(WatchedObjectChangedParameters),
}

pub trait StorageClientObserver {
    fn handle_storage_event(service_handle: ServiceHandle, channel_handle: ChannelHandle, event: StorageClientEvent);
}

pub struct StorageClient<'a, T: StorageClientObserver + PartialEq> {
    channel_handle: ChannelHandle,
    channel: StorageChannel,
    observers: Vec<&'a T>,
}

impl<'a, T: StorageClientObserver + PartialEq> StorageClient<'a, T> {
    pub fn connect_first(process: &mut StormProcess<Self, Self>) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("storage", None, None, None)?;
        let channel = unsafe { StorageChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
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

    pub fn get_capabilities(&self, process: &StormProcess) -> Result<FromChannel<&GetCapabilitiesReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::GetCapabilitiesParameters as u64, false);
            self.channel.commit_message(0);
        }

        process.wait_for_channel_message(self.channel_handle, MessageIds::GetCapabilitiesReturns as u64, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(MessageIds::GetCapabilitiesReturns as u64) {
                let payload = ChannelMessageHeader::get_payload_address(message);
                GetCapabilitiesReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut GetCapabilitiesReturns;
                Ok(FromChannel::new(payload.as_ref().unwrap()))
            }
            else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn list_objects(&self, process: &StormProcess, parameters: &ListObjectsParameters) -> Result<FromChannel<&ListObjectsReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::ListObjectsParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::ListObjectsParameters as u64);
        }

        process.wait_for_channel_message(self.channel_handle, MessageIds::ListObjectsReturns as u64, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(MessageIds::ListObjectsReturns as u64) {
                let payload = ChannelMessageHeader::get_payload_address(message);
                ListObjectsReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut ListObjectsReturns;
                Ok(FromChannel::new(payload.as_ref().unwrap()))
            }
            else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn lock_object(&self, process: &StormProcess, parameters: &LockObjectParameters) -> Result<FromChannel<&LockObjectReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::LockObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::LockObjectParameters as u64);
        }

        process.wait_for_channel_message(self.channel_handle, MessageIds::LockObjectReturns as u64, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(MessageIds::LockObjectReturns as u64) {
                let payload = ChannelMessageHeader::get_payload_address(message);
                LockObjectReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut LockObjectReturns;
                Ok(FromChannel::new(payload.as_ref().unwrap()))
            }
            else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn unlock_object(&self, parameters: &UnlockObjectParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::UnlockObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::UnlockObjectParameters as u64);
        }
    }

    pub fn read_object(&self, process: &StormProcess, parameters: &ReadObjectParameters) -> Result<FromChannel<&ReadObjectReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::ReadObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::ReadObjectParameters as u64);
        }

        process.wait_for_channel_message(self.channel_handle, MessageIds::ReadObjectReturns as u64, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(MessageIds::ReadObjectReturns as u64) {
                let payload = ChannelMessageHeader::get_payload_address(message);
                ReadObjectReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut ReadObjectReturns;
                Ok(FromChannel::new(payload.as_ref().unwrap()))
            }
            else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn write_object(&self, parameters: &WriteObjectParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WriteObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::WriteObjectParameters as u64);
        }
    }

    pub fn watch_object(&self, process: &StormProcess, parameters: &WatchObjectParameters) -> Result<FromChannel<&WatchObjectReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WatchObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::WatchObjectParameters as u64);
        }

        process.wait_for_channel_message(self.channel_handle, MessageIds::WatchObjectReturns as u64, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(MessageIds::WatchObjectReturns as u64) {
                let payload = ChannelMessageHeader::get_payload_address(message);
                WatchObjectReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut WatchObjectReturns;
                Ok(FromChannel::new(payload.as_ref().unwrap()))
            }
            else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn unwatch_object(&self, parameters: &UnwatchObjectParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::UnwatchObjectParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::UnwatchObjectParameters as u64);
        }
    }

}


