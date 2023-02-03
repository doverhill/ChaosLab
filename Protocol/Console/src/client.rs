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
use crate::MessageIds;
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
    fn handle_console_event(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, event: ConsoleClientEvent);
}

pub struct ConsoleClient<'a, T: ConsoleClientObserver> {
    channel_handle: ChannelHandle,
    channel: ConsoleChannel,
    observers: Vec<&'a mut T>,
}

impl<'a, T: ConsoleClientObserver> ConsoleClient<'a, T> {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("console", None, None, None)?;
        let channel = unsafe { ConsoleChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
            observers: Vec::new(),
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: StormEvent, observer: &impl ConsoleClientObserver) {
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
            StormProcess::send_channel_message(self.channel_handle, MessageIds::SetTextColorParameters as u64);
        }
    }

    pub fn move_text_cursor(&self, parameters: &MoveTextCursorParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::MoveTextCursorParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::MoveTextCursorParameters as u64);
        }
    }

    pub fn draw_image_patch(&self, parameters: &DrawImagePatchParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::DrawImagePatchParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::DrawImagePatchParameters as u64);
        }
    }

    pub fn write_text(&self, parameters: &WriteTextParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WriteTextParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::WriteTextParameters as u64);
        }
    }

    pub fn write_objects(&self, parameters: &WriteObjectsParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::WriteObjectsParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::WriteObjectsParameters as u64);
        }
    }

}


