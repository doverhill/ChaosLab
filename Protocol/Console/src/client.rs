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
use crate::channel::{ConsoleChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::vec::Vec;

pub enum ConsoleClientEvent {
    KeyPressed(KeyPressedParameters),
    KeyReleased(KeyReleasedParameters),
    PointerMoved(PointerMovedParameters),
    PointerPressed(PointerPressedParameters),
    PointerReleased(PointerReleasedParameters),
    SizeChanged(SizeChangedParameters),
}

pub trait ConsoleClientObserver {
    fn handle_console_event(&mut self, channel_handle: ChannelHandle, event: ConsoleClientEvent);
}

pub struct ConsoleClient {
    channel_handle: ChannelHandle,
    channel: ConsoleChannel,
}

impl ConsoleClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("console", None, None, None, 4096)?;
        let channel = unsafe { ConsoleChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl ConsoleClientObserver) {
        match event {
            StormEvent::ChannelSignalled(channel_handle) => {
                if *channel_handle == self.channel_handle {
                    println!("ConsoleClient: got event");
                    // observer.handle_console_event(*channel_handle, event);
                }
            }
            _ => {}
        }
    }

    pub fn get_capabilities(&self, process: &StormProcess) -> Result<FromChannel<&GetCapabilitiesReturns>, StormError> {
        unsafe {
            let message = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, false);
            self.channel.commit_message(0);
        }

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        unsafe {
            if let Some(message) = self.channel.find_specific_message(GET_CAPABILITIES_RETURNS) {
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
            let message = self.channel.prepare_message(SET_TEXT_COLOR_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::signal_channel(self.channel_handle);
        }
    }

    pub fn move_text_cursor(&self, parameters: &MoveTextCursorParameters) {
        unsafe {
            let message = self.channel.prepare_message(MOVE_TEXT_CURSOR_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::signal_channel(self.channel_handle);
        }
    }

    pub fn draw_image_patch(&self, parameters: &DrawImagePatchParameters) {
        unsafe {
            let message = self.channel.prepare_message(DRAW_IMAGE_PATCH_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::signal_channel(self.channel_handle);
        }
    }

    pub fn write_text(&self, parameters: &WriteTextParameters) {
        unsafe {
            let message = self.channel.prepare_message(WRITE_TEXT_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::signal_channel(self.channel_handle);
        }
    }

    pub fn write_objects(&self, parameters: &WriteObjectsParameters) {
        unsafe {
            let message = self.channel.prepare_message(WRITE_OBJECTS_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::signal_channel(self.channel_handle);
        }
    }

}


