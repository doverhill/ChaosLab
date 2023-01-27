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

pub struct TornadoClient {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_component_clicked: Option<Box<dyn FnMut()>>,
}

impl TornadoClient {
    pub fn connect_first(process: &mut StormProcess) -> Option<Self> {
    }

    pub fn set_render_tree() {
    }

    pub fn on_component_clicked() {
    }

}


