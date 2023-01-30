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

pub struct TornadoClient {
    channel: TornadoChannel,
    on_component_clicked: Option<Box<dyn Fn(ChannelHandle)>>,
}

impl TornadoClient {
    pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError> {
        let channel_handle = process.connect_to_service("tornado", None, None, None)?;
        let channel = unsafe { TornadoChannel::new(process.get_channel_address(channel_handle).unwrap(), false) };
        Ok(Self {
            channel: channel,
            on_component_clicked: None,
        })
    }

    pub fn set_render_tree(&self, parameters: SetRenderTreeParameters) {
        unsafe {
            let address = self.channel.prepare_message(MessageIds::SetRenderTreeParameters as u64, false);
            let size = parameters.write_at(address);
            self.channel.commit_message(size);
        }
    }

    pub fn on_component_clicked(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>) {
        self.on_component_clicked = handler;
    }

}


