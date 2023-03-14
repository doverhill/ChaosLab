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

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{StorageChannel, ChannelMessageHeader, FromChannel, Coalesce};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;

pub enum StorageClientEvent {
}

pub enum StorageClientChannelEvent {
    ServerDisconnected(ChannelHandle),
    ServerEvent(ChannelHandle, StorageClientEvent),
}

pub struct StorageClient {
    current_event: Option<StormEvent>,
    channel_handle: ChannelHandle,
    channel: StorageChannel,
}

impl StorageClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("storage", None, None, None, 1048576)?;
        let channel = StorageChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            current_event: None,
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &StormProcess) -> Option<StorageClientChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if channel_handle == self.channel_handle {
                        println!("StorageClient: server disconnected");
                        Some(StorageClientChannelEvent::ServerDisconnected(channel_handle))
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
                                    _ => { panic!("StorageClient: Unknown message received"); }
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
                _ => { panic!("StorageClient: Unexpected storm event type"); }
            }
        }
        else {
            None
        }
    }

    pub fn get_capabilities(&mut self, process: &StormProcess) -> Result<FromChannel<GetCapabilitiesReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, Coalesce::Never);
        self.channel.commit_message(0);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetCapabilitiesReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn read(&mut self, process: &StormProcess, parameters: &ReadParameters) -> Result<FromChannel<ReadReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(READ_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { ReadReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn write(&mut self, parameters: &WriteParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

}


