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
use crate::channel::{ConsoleChannel, ChannelMessageHeader, Coalesce};
use crate::from_client::*;
use crate::from_server::*;
use crate::channel::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;

pub enum ConsoleServerRequest {
    GetConsoleCapabilities,
    WriteConsoleText(FromChannel<WriteConsoleTextParameters>),
    DrawImagePatch(FromChannel<DrawImagePatchParameters>),
    DrawPixelDebug(FromChannel<DrawPixelDebugParameters>),
}

pub enum ConsoleServerChannelEvent {
    ClientConnected(ServiceHandle, ChannelHandle),
    ClientDisconnected(ServiceHandle, ChannelHandle),
    ClientRequest(ServiceHandle, ChannelHandle, u64, ConsoleServerRequest),
}

pub struct ConsoleServer {
    current_event: Option<StormEvent>,
    pub service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, ConsoleChannel>,
}

impl ConsoleServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("console", vendor_name, device_name, device_id)?;
        Ok(Self {
            current_event: None,
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &mut StormProcess) -> Option<ConsoleServerChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ServiceConnected(service_handle, channel_handle) => {
                    self.current_event = None;
                    if service_handle == self.service_handle {
                        println!("ConsoleServer: client connected");
                        process.initialize_channel(channel_handle, 1048576);
                        let channel = ConsoleChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), true);
                        self.channels.insert(channel_handle, channel);
                        Some(ConsoleServerChannelEvent::ClientConnected(service_handle, channel_handle))
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
                                    GET_CONSOLE_CAPABILITIES_PARAMETERS => {
                                        channel.unlink_message(message, false);
                                        Some(ConsoleServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, ConsoleServerRequest::GetConsoleCapabilities))
                                    },
                                    WRITE_CONSOLE_TEXT_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        WriteConsoleTextParameters::reconstruct_at_inline(address);
                                        let request = ConsoleServerRequest::WriteConsoleText(FromChannel::new(channel.rx_channel_address, message));
                                        Some(ConsoleServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    DRAW_IMAGE_PATCH_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        DrawImagePatchParameters::reconstruct_at_inline(address);
                                        let request = ConsoleServerRequest::DrawImagePatch(FromChannel::new(channel.rx_channel_address, message));
                                        Some(ConsoleServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    DRAW_PIXEL_DEBUG_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        DrawPixelDebugParameters::reconstruct_at_inline(address);
                                        let request = ConsoleServerRequest::DrawPixelDebug(FromChannel::new(channel.rx_channel_address, message));
                                        Some(ConsoleServerChannelEvent::ClientRequest(self.service_handle, channel_handle, (*message).call_id, request))
                                    },
                                    _ => { panic!("ConsoleServer: Unknown message received"); }
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
                        println!("ConsoleServer: client disconnected");
                        Some(ConsoleServerChannelEvent::ClientDisconnected(self.service_handle, channel_handle))
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

    pub fn key_pressed(&mut self, channel_handle: ChannelHandle, parameters: &KeyPressedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(KEY_PRESSED_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn key_released(&mut self, channel_handle: ChannelHandle, parameters: &KeyReleasedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(KEY_RELEASED_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn character_input(&mut self, channel_handle: ChannelHandle, parameters: &CharacterInputParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(CHARACTER_INPUT_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn pointer_moved(&mut self, channel_handle: ChannelHandle, parameters: &PointerMovedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_MOVED_PARAMETERS, Coalesce::Consecutive);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn pointer_pressed(&mut self, channel_handle: ChannelHandle, parameters: &PointerPressedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_PRESSED_PARAMETERS, Coalesce::Never);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }

    pub fn pointer_released(&mut self, channel_handle: ChannelHandle, parameters: &PointerReleasedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_RELEASED_PARAMETERS, Coalesce::Never);
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

    pub fn get_console_capabilities_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &GetConsoleCapabilitiesReturns) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(GET_CONSOLE_CAPABILITIES_RETURNS, Coalesce::Never);
            unsafe { (*message).call_id = call_id };
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle).unwrap();
        }
    }
}


