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

pub enum ConsoleClientEvent<'a> {
    KeyPressed(&'a KeyPressedParameters),
    KeyReleased(&'a KeyReleasedParameters),
    PointerMoved(&'a PointerMovedParameters),
    PointerPressed(&'a PointerPressedParameters),
    PointerReleased(&'a PointerReleasedParameters),
    SizeChanged(&'a SizeChangedParameters),
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
        let channel = ConsoleChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl ConsoleClientObserver) {
        match event {
            StormEvent::ChannelSignalled(channel_handle) => {
                if *channel_handle == self.channel_handle {
                    while let Some(message) = self.channel.find_message() {
                        unsafe {
                            match (*message).message_id {
                                KEY_PRESSED_PARAMETERS =>  {
                                    println!("got KEY_PRESSED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    KeyPressedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const KeyPressedParameters;
                                    let request = ConsoleClientEvent::KeyPressed(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                KEY_RELEASED_PARAMETERS =>  {
                                    println!("got KEY_RELEASED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    KeyReleasedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const KeyReleasedParameters;
                                    let request = ConsoleClientEvent::KeyReleased(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                POINTER_MOVED_PARAMETERS =>  {
                                    println!("got POINTER_MOVED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    PointerMovedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const PointerMovedParameters;
                                    let request = ConsoleClientEvent::PointerMoved(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                POINTER_PRESSED_PARAMETERS =>  {
                                    println!("got POINTER_PRESSED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    PointerPressedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const PointerPressedParameters;
                                    let request = ConsoleClientEvent::PointerPressed(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                POINTER_RELEASED_PARAMETERS =>  {
                                    println!("got POINTER_RELEASED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    PointerReleasedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const PointerReleasedParameters;
                                    let request = ConsoleClientEvent::PointerReleased(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                SIZE_CHANGED_PARAMETERS =>  {
                                    println!("got SIZE_CHANGED_PARAMETERS message");
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    println!("found message at {:p}", address);
                                    SizeChangedParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const SizeChangedParameters;
                                    let request = ConsoleClientEvent::SizeChanged(parameters.as_ref().unwrap());
                                    observer.handle_console_event(*channel_handle, request);
                                    self.channel.unlink_message(message, false);
                                }
                                _ => {}
                            }
                        }
                    }
                    // observer.handle_console_event(*channel_handle, event);
                }
            }
            _ => {}
        }
    }

    pub fn get_capabilities(&mut self, process: &StormProcess) -> Result<FromChannel<&GetCapabilitiesReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, false);
        self.channel.commit_message(0);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetCapabilitiesReturns::reconstruct_at_inline(payload); }
            let payload = payload as *mut GetCapabilitiesReturns;
            Ok(FromChannel::new(&self.channel, message, unsafe { payload.as_ref().unwrap() }))
        }
        else {
            Err(StormError::NotFound)
        }
    }

    pub fn set_text_color(&mut self, parameters: &SetTextColorParameters) {
        let (call_id, message) = self.channel.prepare_message(SET_TEXT_COLOR_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn move_text_cursor(&mut self, parameters: &MoveTextCursorParameters) {
        let (call_id, message) = self.channel.prepare_message(MOVE_TEXT_CURSOR_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn draw_image_patch(&mut self, parameters: &DrawImagePatchParameters) {
        let (call_id, message) = self.channel.prepare_message(DRAW_IMAGE_PATCH_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn write_text(&mut self, parameters: &WriteTextParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_TEXT_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

    pub fn write_objects(&mut self, parameters: &WriteObjectsParameters) {
        let (call_id, message) = self.channel.prepare_message(WRITE_OBJECTS_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

}


