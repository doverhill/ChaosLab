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
use crate::enums::*;

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{DataChannel, ChannelMessageHeader, FromChannel, Coalesce};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;

pub enum DataClientEvent {
    Characters(FromChannel<CharactersParameters>),
    Commands(FromChannel<CommandsParameters>),
    SizeChanged(FromChannel<SizeChangedParameters>),
}

pub enum DataClientChannelEvent {
    ServerDisconnected(ChannelHandle),
    ServerEvent(ChannelHandle, DataClientEvent),
}

pub struct DataClient {
    current_event: Option<StormEvent>,
    channel_handle: ChannelHandle,
    channel: DataChannel,
}

impl DataClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("data", None, None, None, 1048576)?;
        let channel = DataChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            current_event: None,
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &StormProcess) -> Option<DataClientChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if channel_handle == self.channel_handle {
                        println!("DataClient: server disconnected");
                        Some(DataClientChannelEvent::ServerDisconnected(channel_handle))
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
                                    CHARACTERS_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        CharactersParameters::reconstruct_at_inline(address);
                                        let request = DataClientEvent::Characters(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(DataClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    COMMANDS_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        CommandsParameters::reconstruct_at_inline(address);
                                        let request = DataClientEvent::Commands(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(DataClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    SIZE_CHANGED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        SizeChangedParameters::reconstruct_at_inline(address);
                                        let request = DataClientEvent::SizeChanged(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(DataClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    _ => { panic!("DataClient: Unknown message received"); }
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
                _ => { panic!("DataClient: Unexpected storm event type"); }
            }
        }
        else {
            None
        }
    }

    pub fn get_data_capabilities(&mut self, process: &StormProcess) -> Result<FromChannel<GetDataCapabilitiesReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(GET_DATA_CAPABILITIES_PARAMETERS, Coalesce::Never);
        self.channel.commit_message(0);
        StormProcess::signal_channel(self.channel_handle)?;

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetDataCapabilitiesReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn set_text_color(&mut self, parameters: &SetTextColorParameters) {
        let (call_id, message) = self.channel.prepare_message(SET_TEXT_COLOR_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn save_text_cursor_position(&mut self) {
        let (call_id, message) = self.channel.prepare_message(SAVE_TEXT_CURSOR_POSITION_PARAMETERS, Coalesce::Never);
        self.channel.commit_message(0);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn load_text_cursor_position(&mut self) {
        let (call_id, message) = self.channel.prepare_message(LOAD_TEXT_CURSOR_POSITION_PARAMETERS, Coalesce::Never);
        self.channel.commit_message(0);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn set_text_cursor_position(&mut self, parameters: &SetTextCursorPositionParameters) {
        let (call_id, message) = self.channel.prepare_message(SET_TEXT_CURSOR_POSITION_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn write_text(&mut self, parameters: &WriteTextParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_TEXT_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

    pub fn write_objects(&mut self, parameters: &WriteObjectsParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_OBJECTS_PARAMETERS, Coalesce::Never);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle).unwrap();
    }

}


