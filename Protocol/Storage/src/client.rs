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
use crate::channel::StorageChannel;
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;

pub struct StorageClient {
    channel: StorageChannel,
    on_watched_object_changed: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl StorageClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("storage", None, None, None)?;
        let channel = unsafe { StorageChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel: channel,
            on_watched_object_changed: None,
        })
    }

    pub fn get_capabilities(&self, process: &StormProcess) -> GetCapabilitiesReturns {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::GetCapabilitiesParameters as u64, false);
            self.channel.commit_message(0);
        }
    }

    pub fn list_objects(&self, process: &StormProcess, parameters: ListObjectsParameters) -> ListObjectsReturns {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::ListObjectsParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn lock_object(&self, process: &StormProcess, parameters: LockObjectParameters) -> LockObjectReturns {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::LockObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn unlock_object(&self, parameters: UnlockObjectParameters) {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::UnlockObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn read_object(&self, process: &StormProcess, parameters: ReadObjectParameters) -> ReadObjectReturns {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::ReadObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn write_object(&self, parameters: WriteObjectParameters) {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::WriteObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn watch_object(&self, process: &StormProcess, parameters: WatchObjectParameters) -> WatchObjectReturns {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::WatchObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn unwatch_object(&self, parameters: UnwatchObjectParameters) {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::UnwatchObjectParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn on_watched_object_changed(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_watched_object_changed = handler;
    }

}


