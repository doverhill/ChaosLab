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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, ServiceObserver, ChannelObserver};
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
    fn handle_tornado_event(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle, event: TornadoClientEvent);
}

pub struct TornadoClient<'a, T: TornadoClientObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {
    channel_handle: ChannelHandle,
    channel: TornadoChannel,
    observers: Vec<&'a T>,
    so: Option<&'a SO>,
    co: Option<&'a CO>,
}

impl<'a, T: TornadoClientObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> TornadoClient<'a, T, SO, CO> {
    pub fn connect_first(process: &mut StormProcess<SO, CO>) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("tornado", None, None, None)?;
        let channel = unsafe { TornadoChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel_handle: channel_handle,
            channel: channel,
            observers: Vec::new(),
            so: None,
            co: None,
        })
    }

    pub fn attach_observer(&mut self, observer: &'a T) {
        self.observers.push(observer);
    }

    pub fn detach_observer(&mut self, observer: &'a T) {
        if let Some(index) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(index);
        }
    }

    pub fn set_render_tree(&self, parameters: &SetRenderTreeParameters) {
        unsafe {
            let message = self.channel.prepare_message(MessageIds::SetRenderTreeParameters as u64, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = parameters.write_at(payload);
            self.channel.commit_message(size);
            StormProcess::<SO, CO>::send_channel_message(self.channel_handle, MessageIds::SetRenderTreeParameters as u64);
        }
    }

}


