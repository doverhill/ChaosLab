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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{DataChannel, ChannelMessageHeader, Coalesce};
use crate::from_client::*;
use crate::from_server::*;
use crate::channel::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;

pub enum DataServerRequest {
    GetDataCapabilities,
    SetTextColor(FromChannel<SetTextColorParameters>),
    SaveTextCursorPosition,
    LoadTextCursorPosition,
    SetTextCursorPosition(FromChannel<SetTextCursorPositionParameters>),
    WriteText(FromChannel<WriteTextParameters>),
    WriteObjects(FromChannel<WriteObjectsParameters>),
}

pub enum DataServerChannelEvent {
    ClientConnected(ServiceHandle, ChannelHandle),
    ClientDisconnected(ServiceHandle, ChannelHandle),
    ClientRequest(ServiceHandle, ChannelHandle, u64, DataServerRequest),
}

pub struct DataServer {
    current_event: Option<StormEvent>,
    pub service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, DataChannel>,
}

impl DataServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("data", vendor_name, device_name, device_id)?;
        Ok(Self {
            current_event: None,
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &mut StormProcess) -> Option<DataServerChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ServiceConnected(service_handle, channel_handle) => {
                    self.current_event = None;
                    if service_handle == self.service_handle {
                        println!("DataServer: client connected");
                        process.initialize_channel(channel_handle, 1048576);
                        let channel = DataChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), true);
                        self.channels.insert(channel_handle, channel);
                        Some(DataServerChannelEvent::ClientConnected(service_handle, channel_handle))
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
                                    GET_DATA_CAPABILITIES_PARAMETERS => {
                                        channel.unlink_message(message, false);
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, DataServerRequest::GetDataCapabilities))
                                    },
                                    SET_TEXT_COLOR_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        SetTextColorParameters::reconstruct_at_inline(address);
                                        let request = DataServerRequest::SetTextColor(FromChannel::new(channel.rx_channel_address, message));
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    SAVE_TEXT_CURSOR_POSITION_PARAMETERS => {
                                        channel.unlink_message(message, false);
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, DataServerRequest::SaveTextCursorPosition))
                                    },
                                    LOAD_TEXT_CURSOR_POSITION_PARAMETERS => {
                                        channel.unlink_message(message, false);
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, DataServerRequest::LoadTextCursorPosition))
                                    },
                                    SET_TEXT_CURSOR_POSITION_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        SetTextCursorPositionParameters::reconstruct_at_inline(address);
                                        let request = DataServerRequest::SetTextCursorPosition(FromChannel::new(channel.rx_channel_address, message));
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    WRITE_TEXT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WriteTextParameters::reconstruct_at_inline(address);
                                        let request = DataServerRequest::WriteText(FromChannel::new(channel.rx_channel_address, message));
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    WRITE_OBJECTS_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WriteObjectsParameters::reconstruct_at_inline(address);
                                        let request = DataServerRequest::WriteObjects(FromChannel::new(channel.rx_channel_address, message));
                                        Some(DataServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    _ => { panic!("DataServer: Unknown message received"); }
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
                        println!("DataServer: client disconnected");
                        Some(DataServerChannelEvent::ClientDisconnected(self.service_handle, channel_handle))
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

    pub fn characters(&mut self, channel_handle: ChannelHandle, parameters: &CharactersParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(CHARACTERS_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn commands(&mut self, channel_handle: ChannelHandle, parameters: &CommandsParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(COMMANDS_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn size_changed(&mut self, channel_handle: ChannelHandle, parameters: &SizeChangedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(SIZE_CHANGED_PARAMETERS, Coalesce::Consecutive);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn get_data_capabilities_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &GetDataCapabilitiesReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(GET_DATA_CAPABILITIES_RETURNS, Coalesce::Never);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
}


