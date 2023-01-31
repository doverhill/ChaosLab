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
use crate::channel::{ConsoleChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;

pub struct ConsoleClient {
    channel_handle: ChannelHandle,
    channel: ConsoleChannel,
    on_key_pressed: Option<Box<dyn Fn(ChannelHandle)>>,
    on_key_released: Option<Box<dyn Fn(ChannelHandle)>>,
    on_pointer_moved: Option<Box<dyn Fn(ChannelHandle)>>,
    on_pointer_pressed: Option<Box<dyn Fn(ChannelHandle)>>,
    on_pointer_released: Option<Box<dyn Fn(ChannelHandle)>>,
    on_size_changed: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl ConsoleClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("console", None, None, None)?;
        let channel = unsafe { ConsoleChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
            on_key_pressed: None,
            on_key_released: None,
            on_pointer_moved: None,
            on_pointer_pressed: None,
            on_pointer_released: None,
            on_size_changed: None,
        })
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

    pub fn set_text_color(&self, parameters: &SetTextColorParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::SetTextColorParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
        }
    }

    pub fn move_text_cursor(&self, parameters: &MoveTextCursorParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::MoveTextCursorParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
        }
    }

    pub fn draw_image_patch(&self, parameters: &DrawImagePatchParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::DrawImagePatchParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
        }
    }

    pub fn write_text(&self, parameters: &WriteTextParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WriteTextParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
        }
    }

    pub fn write_objects(&self, parameters: &WriteObjectsParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WriteObjectsParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
        }
    }

    pub fn on_key_pressed(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_key_pressed = Some(Box::new(handler));
    }

    pub fn clear_on_key_pressed(&mut self) {
        self.on_key_pressed = None;
    }

    pub fn on_key_released(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_key_released = Some(Box::new(handler));
    }

    pub fn clear_on_key_released(&mut self) {
        self.on_key_released = None;
    }

    pub fn on_pointer_moved(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_pointer_moved = Some(Box::new(handler));
    }

    pub fn clear_on_pointer_moved(&mut self) {
        self.on_pointer_moved = None;
    }

    pub fn on_pointer_pressed(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_pointer_pressed = Some(Box::new(handler));
    }

    pub fn clear_on_pointer_pressed(&mut self) {
        self.on_pointer_pressed = None;
    }

    pub fn on_pointer_released(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_pointer_released = Some(Box::new(handler));
    }

    pub fn clear_on_pointer_released(&mut self) {
        self.on_pointer_released = None;
    }

    pub fn on_size_changed(&mut self, handler: impl Fn(ChannelHandle) + 'static) {
        self.on_size_changed = Some(Box::new(handler));
    }

    pub fn clear_on_size_changed(&mut self) {
        self.on_size_changed = None;
    }

}


