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
use crate::channel::{TornadoChannel, ChannelMessageHeader, FromChannel};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::vec::Vec;

pub enum TornadoClientEvent {
    ComponentClicked(ComponentClickedParameters),
}

pub trait TornadoClientObserver {
    fn handle_tornado_event(&mut self, channel_handle: ChannelHandle, event: TornadoClientEvent);
}

pub struct TornadoClient {
    channel_handle: ChannelHandle,
    channel: TornadoChannel,
}

impl TornadoClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("tornado", None, None, None, 4096)?;
        let channel = TornadoChannel::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl TornadoClientObserver) {
        match event {
            StormEvent::ChannelSignalled(channel_handle) => {
                if *channel_handle == self.channel_handle {
                    // observer.handle_tornado_event(*channel_handle, event);
                }
            }
            _ => {}
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


