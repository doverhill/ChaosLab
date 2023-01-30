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
use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError};
use uuid::Uuid;
use crate::channel::TornadoChannel;
use crate::from_client::*;
use crate::from_server::*;
use crate::MessageIds;
use alloc::collections::BTreeMap;

pub struct TornadoServer {
    channels: BTreeMap<ChannelHandle, TornadoChannel>,
    on_client_connected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_client_disconnected: Option<Box<dyn Fn(ChannelHandle)>>,
    on_set_render_tree: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl TornadoServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("tornado", vendor_name, device_name, device_id)?;
        Ok(Self {
            channels: BTreeMap::new(),
            on_client_connected: None,
            on_client_disconnected: None,
            on_set_render_tree: None,
        })
    }

    pub fn on_client_connected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_connected = handler;
    }

    pub fn on_client_disconnected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_client_disconnected = handler;
    }

    pub fn component_clicked(&self, channel_handle: ChannelHandle, parameters: ComponentClickedParameters) {
        if let Some(channel) = self.channels.get(&channel_handle) {
            unsafe {
                let address = channel.prepare_message(MessageIds::ComponentClickedParameters as u64, false);
                let size = parameters.write_at(address);
                channel.commit_message(size);
            }
        }
    }

    pub fn on_set_render_tree(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_set_render_tree = handler;
    }

}


