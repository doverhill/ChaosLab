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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError};
use uuid::Uuid;
use crate::channel::ConsoleChannel;
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;

pub struct ConsoleServer {
    channels: BTreeMap<ChannelHandle, ConsoleChannel>,
    on_client_connected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_client_disconnected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_get_capabilities: Option<Box<dyn Fn(ChannelHandle)>>,
    on_set_text_color: Option<Box<dyn Fn(ChannelHandle)>>,
    on_move_text_cursor: Option<Box<dyn Fn(ChannelHandle)>>,
    on_draw_image_patch: Option<Box<dyn Fn(ChannelHandle)>>,
    on_write_text: Option<Box<dyn Fn(ChannelHandle)>>,
    on_write_objects: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl ConsoleServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("console", vendor_name, device_name, device_id)?;
        Ok(Self {
            channels: BTreeMap::new(),
            on_client_connected: None,
            on_client_disconnected: None,
            on_get_capabilities: None,
            on_set_text_color: None,
            on_move_text_cursor: None,
            on_draw_image_patch: None,
            on_write_text: None,
            on_write_objects: None,
        })
    }

    pub fn on_client_connected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_connected = handler;
    }

    pub fn on_client_disconnected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_disconnected = handler;
    }

    pub fn key_pressed(&self, channel_handle: ChannelHandle, parameters: KeyPressedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::KeyPressedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn key_released(&self, channel_handle: ChannelHandle, parameters: KeyReleasedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::KeyReleasedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn pointer_moved(&self, channel_handle: ChannelHandle, parameters: PointerMovedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::PointerMovedParameters as u64, true);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn pointer_pressed(&self, channel_handle: ChannelHandle, parameters: PointerPressedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::PointerPressedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn pointer_released(&self, channel_handle: ChannelHandle, parameters: PointerReleasedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::PointerReleasedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn size_changed(&self, channel_handle: ChannelHandle, parameters: SizeChangedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::SizeChangedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn on_get_capabilities(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_get_capabilities = handler;
    }

    pub fn on_set_text_color(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_set_text_color = handler;
    }

    pub fn on_move_text_cursor(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_move_text_cursor = handler;
    }

    pub fn on_draw_image_patch(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_draw_image_patch = handler;
    }

    pub fn on_write_text(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_write_text = handler;
    }

    pub fn on_write_objects(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_write_objects = handler;
    }

}

