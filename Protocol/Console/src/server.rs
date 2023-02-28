#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

use alloc::boxed::Box;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{ConsoleChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum ConsoleServerRequest<'a> {
    GetCapabilities,
    SetTextColor(&'a SetTextColorParameters),
    MoveTextCursor(&'a MoveTextCursorParameters),
    DrawImagePatch(&'a DrawImagePatchParameters),
    WriteText(&'a WriteTextParameters),
    WriteObjects(&'a WriteObjectsParameters),
}

pub trait ConsoleServerObserver {
    fn handle_console_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_console_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_console_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: ConsoleServerRequest);
}

pub struct ConsoleServer {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, ConsoleChannel>,
}

impl ConsoleServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("console", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn process_event(&mut self, process: &mut StormProcess, event: &StormEvent, observer: &mut impl ConsoleServerObserver) {
        match event {
            StormEvent::ServiceConnected(service_handle, channel_handle) => {
                if *service_handle == self.service_handle {
                    println!("ConsoleServer: client connected");
                    process.initialize_channel(*channel_handle, 4096);
                    let channel = ConsoleChannel::new(process.get_channel_address(*channel_handle, 0).unwrap(), process.get_channel_address(*channel_handle, 1).unwrap(), true);
                    self.channels.insert(*channel_handle, channel);
                    observer.handle_console_client_connected(*service_handle, *channel_handle);
                }
            }
            StormEvent::ChannelSignalled(channel_handle) => {
                if let Some(channel) = self.channels.get(&channel_handle) {
                    while let Some(message) = channel.find_message() {
                        unsafe {
                            match (*message).message_id {
                                GET_CAPABILITIES_PARAMETERS =>  {
                                    println!("got GET_CAPABILITIES_PARAMETERS message");
                                    observer.handle_console_request(self.service_handle, *channel_handle, ConsoleServerRequest::GetCapabilities);
                                    channel.unlink_message(message, false);
                                }
                                SET_TEXT_COLOR_PARAMETERS =>  {
                                    println!("got SET_TEXT_COLOR_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    SetTextColorParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const SetTextColorParameters;
                                    let request = ConsoleServerRequest::SetTextColor(parameters.as_ref().unwrap());
                                    observer.handle_console_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                MOVE_TEXT_CURSOR_PARAMETERS =>  {
                                    println!("got MOVE_TEXT_CURSOR_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    MoveTextCursorParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const MoveTextCursorParameters;
                                    let request = ConsoleServerRequest::MoveTextCursor(parameters.as_ref().unwrap());
                                    observer.handle_console_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                DRAW_IMAGE_PATCH_PARAMETERS =>  {
                                    println!("got DRAW_IMAGE_PATCH_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    DrawImagePatchParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const DrawImagePatchParameters;
                                    let request = ConsoleServerRequest::DrawImagePatch(parameters.as_ref().unwrap());
                                    observer.handle_console_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                WRITE_TEXT_PARAMETERS =>  {
                                    println!("got WRITE_TEXT_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    WriteTextParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const WriteTextParameters;
                                    let request = ConsoleServerRequest::WriteText(parameters.as_ref().unwrap());
                                    observer.handle_console_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                WRITE_OBJECTS_PARAMETERS =>  {
                                    println!("got WRITE_OBJECTS_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    WriteObjectsParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const WriteObjectsParameters;
                                    let request = ConsoleServerRequest::WriteObjects(parameters.as_ref().unwrap());
                                    observer.handle_console_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            StormEvent::ChannelDestroyed(channel_handle) => {
                if let Some(_) = self.channels.get(&channel_handle) {
                    println!("ConsoleServer: client disconnected");
                    observer.handle_console_client_disconnected(self.service_handle, *channel_handle);
                }
            }
        }
    }

    pub fn key_pressed(&mut self, channel_handle: ChannelHandle, parameters: KeyPressedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(KEY_PRESSED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn key_released(&mut self, channel_handle: ChannelHandle, parameters: KeyReleasedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(KEY_RELEASED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn pointer_moved(&mut self, channel_handle: ChannelHandle, parameters: PointerMovedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_MOVED_PARAMETERS, true);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn pointer_pressed(&mut self, channel_handle: ChannelHandle, parameters: PointerPressedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_PRESSED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn pointer_released(&mut self, channel_handle: ChannelHandle, parameters: PointerReleasedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(POINTER_RELEASED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

    pub fn size_changed(&mut self, channel_handle: ChannelHandle, parameters: SizeChangedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(SIZE_CHANGED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

}


