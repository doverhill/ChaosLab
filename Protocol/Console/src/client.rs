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
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};
use uuid::Uuid;
use crate::channel::{ConsoleChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::vec::Vec;

pub enum ConsoleClientEvent {
    KeyPressed(FromChannel<KeyPressedParameters>),
    KeyReleased(FromChannel<KeyReleasedParameters>),
    PointerMoved(FromChannel<PointerMovedParameters>),
    PointerPressed(FromChannel<PointerPressedParameters>),
    PointerReleased(FromChannel<PointerReleasedParameters>),
    SizeChanged(FromChannel<SizeChangedParameters>),
}

pub enum ConsoleClientChannelEvent {
    ServerDisconnected(ChannelHandle),
    ServerEvent(ChannelHandle, ConsoleClientEvent),
}

pub struct ConsoleClient {
    current_event: Option<StormEvent>,
    channel_handle: ChannelHandle,
    channel: ConsoleChannel,
}

impl ConsoleClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("console", None, None, None, 4096)?;
        let channel = ConsoleChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            current_event: None,
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &StormProcess) -> Option<ConsoleClientChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if channel_handle == self.channel_handle {
                        Some(ConsoleClientChannelEvent::ServerDisconnected(channel_handle))
                    }
                    else {
                        None
                    }
                }
                StormEvent::ChannelSignalled(channel_handle) => {
                    if channel_handle == self.channel_handle {
                        if let Some(message) = self.channel.find_message() {
                            unsafe {
                                match (*message).message_id {
                                    KEY_PRESSED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        KeyPressedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::KeyPressed(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    KEY_RELEASED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        KeyReleasedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::KeyReleased(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    POINTER_MOVED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        PointerMovedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::PointerMoved(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    POINTER_PRESSED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        PointerPressedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::PointerPressed(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    POINTER_RELEASED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        PointerReleasedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::PointerReleased(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    SIZE_CHANGED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        SizeChangedParameters::reconstruct_at_inline(address);
                                        let request = ConsoleClientEvent::SizeChanged(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(ConsoleClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    _ => { panic!("ConsoleClient: Unknown message received"); }
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
                _ => { panic!("ConsoleClient: Unexpected storm event type"); }
            }
        }
        else {
            None
        }
    }

    pub fn get_capabilities(&mut self, process: &StormProcess) -> Result<FromChannel<GetCapabilitiesReturns>, StormError> {
        let (call_id, message) = self.channel.prepare_message(GET_CAPABILITIES_PARAMETERS, false);
        self.channel.commit_message(0);
        StormProcess::signal_channel(self.channel_handle);

        process.wait_for_channel_signal(self.channel_handle, 1000)?;

        if let Some(message) = self.channel.find_specific_message(call_id) {
            let payload = ChannelMessageHeader::get_payload_address(message);
            unsafe { GetCapabilitiesReturns::reconstruct_at_inline(payload); }
            Ok(FromChannel::new(self.channel.rx_channel_address, message))
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


