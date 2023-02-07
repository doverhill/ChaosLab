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
use crate::MessageIds;
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
        let channel_handle = process.connect_to_service("storage", None, None, None, 0)?;
        let channel = unsafe { StorageChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl StorageClientObserver) {
        match event {
            StormEvent::ChannelMessaged(channel_handle, message_id) => {
                if *channel_handle == self.channel_handle {
                    println!("StorageClient: got event");
                    // observer.handle_storage_event(*channel_handle, event);
                }
            }
            _ => {}
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


