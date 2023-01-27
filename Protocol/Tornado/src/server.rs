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
use library_chaos::{StormProcess, StormHandle};
use uuid::Uuid;

pub struct TornadoServer {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_set_render_tree: Option<Box<dyn FnMut()>>,
}

impl TornadoServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<Self> {
    }

    pub fn component_clicked() {
    }

    pub fn on_set_render_tree() {
    }

}


