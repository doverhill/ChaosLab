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
use crate::channel::{TornadoChannel, ChannelMessageHeader};
use crate::from_client::*;
use crate::from_server::*;
use crate::message_ids::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum TornadoServerRequest<'a> {
    SetRenderTree(&'a SetRenderTreeParameters),
}

pub trait TornadoServerObserver {
    fn handle_tornado_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_tornado_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
    fn handle_tornado_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: TornadoServerRequest);
}

pub struct TornadoServer {
    service_handle: ServiceHandle,
    channels: BTreeMap<ChannelHandle, TornadoChannel>,
}

impl TornadoServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError> {
        let service_handle = process.create_service("tornado", vendor_name, device_name, device_id)?;
        Ok(Self {
            service_handle: service_handle,
            channels: BTreeMap::new(),
        })
    }

    pub fn process_event(&mut self, process: &mut StormProcess, event: &StormEvent, observer: &mut impl TornadoServerObserver) {
        match event {
            StormEvent::ServiceConnected(service_handle, channel_handle) => {
                if *service_handle == self.service_handle {
                    println!("TornadoServer: client connected");
                    process.initialize_channel(*channel_handle, 4096);
                    let channel = TornadoChannel::new(process.get_channel_address(*channel_handle, 0).unwrap(), process.get_channel_address(*channel_handle, 1).unwrap(), true);
                    self.channels.insert(*channel_handle, channel);
                    observer.handle_tornado_client_connected(*service_handle, *channel_handle);
                }
            }
            StormEvent::ChannelSignalled(channel_handle) => {
                if let Some(channel) = self.channels.get(&channel_handle) {
                    while let Some(message) = channel.find_message() {
                        unsafe {
                            match (*message).message_id {
                                SET_RENDER_TREE_PARAMETERS =>  {
                                    let address = ChannelMessageHeader::get_payload_address(message);
                                    SetRenderTreeParameters::reconstruct_at_inline(address);
                                    let parameters = address as *const SetRenderTreeParameters;
                                    let request = TornadoServerRequest::SetRenderTree(parameters.as_ref().unwrap());
                                    observer.handle_tornado_request(self.service_handle, *channel_handle, request);
                                    channel.unlink_message(message, false);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            StormEvent::ChannelDestroyed(channel_handle) => {
                if let Some(_) = self.channels.get(&channel_handle) {
                    println!("TornadoServer: client disconnected");
                    observer.handle_tornado_client_disconnected(self.service_handle, *channel_handle);
                }
            }
        }
    }

    pub fn component_clicked(&mut self, channel_handle: ChannelHandle, parameters: ComponentClickedParameters) {
        if let Some(channel) = self.channels.get_mut(&channel_handle) {
            let (_, message) = channel.prepare_message(COMPONENT_CLICKED_PARAMETERS, false);
            let payload = ChannelMessageHeader::get_payload_address(message);
            let size = unsafe { parameters.write_at(payload) };
            channel.commit_message(size);
            StormProcess::signal_channel(channel_handle);
        }
    }

}


