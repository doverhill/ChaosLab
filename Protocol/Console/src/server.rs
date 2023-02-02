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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, ServiceObserver, ChannelObserver};
use uuid::Uuid;
use crate::channel::{ConsoleChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum ConsoleServerRequest {
    GetCapabilities,
    SetTextColor(SetTextColorParameters),
    MoveTextCursor(MoveTextCursorParameters),
    DrawImagePatch(DrawImagePatchParameters),
    WriteText(WriteTextParameters),
    WriteObjects(WriteObjectsParameters),
}

pub trait ConsoleServerObserver {
    fn handle_console_client_connected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_console_client_disconnected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_console_request(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: ConsoleServerRequest);
}

pub struct ConsoleServer<'a, T: ConsoleServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, ConsoleChannel>,
    observers: Vec<&'a T>,
    so: Option<&'a SO>,
    co: Option<&'a CO>,
}

impl<'a, T: ConsoleServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> ConsoleServer<'a, T, SO, CO> {
    pub fn create(process: &mut StormProcess<SO, CO>, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("console", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
            observers: Vec::new(),
            so: None,
            co: None,
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

    pub fn key_pressed(&self, channel_handle: ChannelHandle, parameters: KeyPressedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::KeyPressedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::KeyPressedParameters as u64);
            }
        }
    }

    pub fn key_released(&self, channel_handle: ChannelHandle, parameters: KeyReleasedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::KeyReleasedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::KeyReleasedParameters as u64);
            }
        }
    }

    pub fn pointer_moved(&self, channel_handle: ChannelHandle, parameters: PointerMovedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::PointerMovedParameters as u64, true);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::PointerMovedParameters as u64);
            }
        }
    }

    pub fn pointer_pressed(&self, channel_handle: ChannelHandle, parameters: PointerPressedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::PointerPressedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::PointerPressedParameters as u64);
            }
        }
    }

    pub fn pointer_released(&self, channel_handle: ChannelHandle, parameters: PointerReleasedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::PointerReleasedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::PointerReleasedParameters as u64);
            }
        }
    }

    pub fn size_changed(&self, channel_handle: ChannelHandle, parameters: SizeChangedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::SizeChangedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::SizeChangedParameters as u64);
            }
        }
    }

}


