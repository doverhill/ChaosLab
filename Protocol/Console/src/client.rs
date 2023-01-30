#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use crate::enums::*;
use crate::types::*;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;

use crate::channel::{ChannelMessageHeader, ConsoleChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::boxed::Box;
use library_chaos::{ChannelHandle, ServiceHandle, StormError, StormProcess};
use uuid::Uuid;

struct GetCapabilitiesReturnsWrapper {
    channel: ConsoleChannel,
    message: *mut ChannelMessageHeader,
    pub value: &'static GetCapabilitiesReturns,
}

impl Drop for GetCapabilitiesReturnsWrapper {
    fn drop(&mut self) {
        unsafe {
            self.channel.unlink_message(self.message, false);
        }
    }
}

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
        let channel = unsafe {
            ConsoleChannel::new(process.get_channel_address(channel_handle).unwrap(), false)
        };
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

    pub fn get_capabilities(
        &self,
        process: &StormProcess,
    ) -> Result<GetCapabilitiesReturnsWrapper, StormError> {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::GetCapabilitiesParameters as u64, false);
            self.channel.commit_message(0);
        }

        process.wait_for_channel_message(
            self.channel_handle,
            MessageIds::GetCapabilitiesReturns as u64,
            1000,
        )?;

        unsafe {
            if let Some(message) = self
                .channel
                .find_specific_message(MessageIds::GetCapabilitiesReturns as u64)
            {
                let payload = ChannelMessageHeader::get_payload_address(message);
                GetCapabilitiesReturns::reconstruct_at_inline(payload);
                let payload = payload as *mut GetCapabilitiesReturns;

                // fixme: make ConsoleChannel methods 'static'
                Ok(GetCapabilitiesReturnsWrapper {
                    channel: self.channel,
                    message: message,
                    value: payload.as_ref().unwrap(),
            })
            } else {
                Err(StormError::NotFound)
            }
        }
    }

    pub fn set_text_color(&self, parameters: SetTextColorParameters) {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::SetTextColorParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn move_text_cursor(&self, parameters: MoveTextCursorParameters) {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::MoveTextCursorParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn draw_image_patch(&self, parameters: DrawImagePatchParameters) {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::DrawImagePatchParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn write_text(&self, parameters: WriteTextParameters) {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::WriteTextParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn write_objects(&self, parameters: WriteObjectsParameters) {
        unsafe {
            let address = self
                .channel
                .prepare_message(MessageIds::WriteObjectsParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn on_key_pressed(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_key_pressed = handler;
    }

    pub fn on_key_released(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_key_released = handler;
    }

    pub fn on_pointer_moved(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_pointer_moved = handler;
    }

    pub fn on_pointer_pressed(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_pointer_pressed = handler;
    }

    pub fn on_pointer_released(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_pointer_released = handler;
    }

    pub fn on_size_changed(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_size_changed = handler;
    }
}
