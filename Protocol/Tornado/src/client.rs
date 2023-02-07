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
use crate::MessageIds;
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
        let channel_handle = process.connect_to_service("tornado", None, None, None, 0)?;
        let channel = unsafe { TornadoChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
        })
    }

    pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl TornadoClientObserver) {
        match event {
            StormEvent::ChannelMessaged(channel_handle, message_id) => {
                if *channel_handle == self.channel_handle {
                    println!("TornadoClient: got event");
                    // observer.handle_tornado_event(*channel_handle, event);
                }
            }
            _ => {}
        }
    }

    pub fn set_render_tree(&self, parameters: &SetRenderTreeParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::SetRenderTreeParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::send_channel_message(self.channel_handle, MessageIds::SetRenderTreeParameters as u64);
        }
    }

}


