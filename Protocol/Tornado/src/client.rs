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
use crate::channel::{TornadoChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::vec::Vec;

pub enum TornadoClientEvent {
    ComponentClicked(FromChannel<ComponentClickedParameters>),
}

pub enum TornadoClientChannelEvent {
    ServerDisconnected(ChannelHandle),
    ServerEvent(ChannelHandle, TornadoClientEvent),
}

pub struct TornadoClient {
    current_event: Option<StormEvent>,
    channel_handle: ChannelHandle,
    channel: TornadoChannel,
}

impl TornadoClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("tornado", None, None, None, 4096)?;
        let channel = TornadoChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            current_event: None,
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn register_event(&mut self, event: StormEvent) {
        self.current_event = Some(event);
    }

    pub fn get_event(&mut self, process: &StormProcess) -> Option<TornadoClientChannelEvent> {
        if let Some(current_event) = self.current_event {
            match current_event {
                StormEvent::ChannelDestroyed(channel_handle) => {
                    self.current_event = None;
                    if channel_handle == self.channel_handle {
                        Some(TornadoClientChannelEvent::ServerDisconnected(channel_handle))
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
                                    COMPONENT_CLICKED_PARAMETERS => {
                                        let address = ChannelMessageHeader::get_payload_address(message);
                                        ComponentClickedParameters::reconstruct_at_inline(address);
                                        let request = TornadoClientEvent::ComponentClicked(FromChannel::new(self.channel.rx_channel_address, message));
                                        Some(TornadoClientChannelEvent::ServerEvent(channel_handle, request))
                                    },
                                    _ => { panic!("TornadoClient: Unknown message received"); }
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
                _ => { panic!("TornadoClient: Unexpected storm event type"); }
            }
        }
        else {
            None
        }
    }

    pub fn set_render_tree(&mut self, parameters: &SetRenderTreeParameters) {
        let (call_id, message) = self.channel.prepare_message(SET_RENDER_TREE_PARAMETERS, false);
        let payload = ChannelMessageHeader::get_payload_address(message);
        let size = unsafe { parameters.write_at(payload) };
        self.channel.commit_message(size);
        StormProcess::signal_channel(self.channel_handle);
    }

}


