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

pub struct ConsoleServer {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_get_capabilities: Option<Box<dyn FnMut()>>,
    on_set_text_color: Option<Box<dyn FnMut()>>,
    on_move_text_cursor: Option<Box<dyn FnMut()>>,
    on_draw_image_patch: Option<Box<dyn FnMut()>>,
    on_write_text: Option<Box<dyn FnMut()>>,
    on_write_objects: Option<Box<dyn FnMut()>>,
}

impl ConsoleServer {
    pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<Self> {
    }

    pub fn key_pressed() {
    }

    pub fn key_released() {
    }

    pub fn pointer_moved() {
    }

    pub fn pointer_pressed() {
    }

    pub fn pointer_released() {
    }

    pub fn size_changed() {
    }

    pub fn on_get_capabilities() {
    }

    pub fn on_set_text_color() {
    }

    pub fn on_move_text_cursor() {
    }

    pub fn on_draw_image_patch() {
    }

    pub fn on_write_text() {
    }

    pub fn on_write_objects() {
    }

}


