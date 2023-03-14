#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use alloc::vec::Vec;
use alloc::string::String;
use crate::types::*;

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{FilesystemChannel, ChannelMessageHeader, FromChannel, Coalesce};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;

pub enum FilesystemClientEvent {
    WatchedObjectChanged(FromChannel<WatchedObjectChangedParameters>),
}

pub enum FilesystemClientChannelEvent {
    ServerDisconnected(ChannelHandle),
    ServerEvent(ChannelHandle, FilesystemClientEvent),
}

pub struct FilesystemClient {
    current_event: Option<StormEvent>,
    channel_handle: ChannelHandle,
    channel: FilesystemChannel,
}

impl FilesystemClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("filesystem", None, None, None, 1048576)?;
        let channel = FilesystemChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            current_event: None,
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &StormProcess) -> Option<FilesystemClientChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if channel_handle == self.channel_handle {
                        println!("FilesystemClient: server disconnected");
                        Some(FilesystemClientChannelEvent::ServerDisconnected(channel_handle))
                    }
                    else {
                        None
                    }
                }
                StormEvent::ChannelSignalled(channel_handle) => {
                    if channel_handle == self.channel_handle {
                        if let Some(message) = self.channel.find_message() {
                            unsafe {
                                match (*message).message_id {
                                    WATCHED_OBJECT_CHANGED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WatchedObjectChangedParameters::reconstruct_at_inline(address);
                                        let request = FilesystemClientEvent::WatchedObjectChanged(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(FilesystemClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    _ => { panic!("FilesystemClient: Unknown message received"); }
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
                _ => { panic!("FilesystemClient: Unexpected storm event type"); }
            }
        }
        else {
            None
        }
    }

    pub fn list_objects(&mut self, process: &StormProcess, parameters: &ListObjectsParameters) -> Result<FromChannel<ListObjectsReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(LIST_OBJECTS_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ListObjectsReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn lock_object(&mut self, process: &StormProcess, parameters: &LockObjectParameters) -> Result<FromChannel<LockObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(LOCK_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { LockObjectReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unlock_object(&mut self, parameters: &UnlockObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(UNLOCK_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn read_object(&mut self, process: &StormProcess, parameters: &ReadObjectParameters) -> Result<FromChannel<ReadObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(READ_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ReadObjectReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn write_object(&mut self, parameters: &WriteObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn watch_object(&mut self, process: &StormProcess, parameters: &WatchObjectParameters) -> Result<FromChannel<WatchObjectReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(WATCH_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { WatchObjectReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn unwatch_object(&mut self, parameters: &UnwatchObjectParameters) {
        let (call_id, message) = self.channel.prepare_message(UNWATCH_OBJECT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

}


