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
use crate::channel::{TornadoChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum TornadoServerRequest {
    SetRenderTree(SetRenderTreeParameters),
}

pub trait TornadoServerObserver {
    fn handle_tornado_client_connected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_tornado_client_disconnected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_tornado_request(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: TornadoServerRequest);
}

pub struct TornadoServer<'a, T: TornadoServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, TornadoChannel>,
    observers: Vec<&'a T>,
    so: Option<&'a SO>,
    co: Option<&'a CO>,
}

impl<'a, T: TornadoServerObserver + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> TornadoServer<'a, T, SO, CO> {
    pub fn create(process: &mut StormProcess<SO, CO>, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("tornado", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
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

    pub fn component_clicked(&self, channel_handle: ChannelHandle, parameters: ComponentClickedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let message = channel.prepare_message(MessageIds::ComponentClickedParameters as u64, false);
                let payload = ChannelMessageHeader::get_payload_address(message);
                let size = parameters.write_at(payload);
                channel.commit_message(size);
                StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::ComponentClickedParameters as u64);
            }
        }
    }

}


