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

pub struct ConsoleClient {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_key_pressed: Option<Box<dyn FnMut()>>,
    on_key_released: Option<Box<dyn FnMut()>>,
    on_pointer_moved: Option<Box<dyn FnMut()>>,
    on_pointer_pressed: Option<Box<dyn FnMut()>>,
    on_pointer_released: Option<Box<dyn FnMut()>>,
    on_size_changed: Option<Box<dyn FnMut()>>,
}

impl ConsoleClient {
    pub fn create(process: &StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<StormHandle> {
    }
    pub fn get_capabilities() {
    }

    pub fn set_text_color() {
    }

    pub fn move_text_cursor() {
    }

    pub fn draw_image_patch() {
    }

    pub fn write_text() {
    }

    pub fn write_objects() {
    }

    pub fn on_key_pressed() {
    }

    pub fn on_key_released() {
    }

    pub fn on_pointer_moved() {
    }

    pub fn on_pointer_pressed() {
    }

    pub fn on_pointer_released() {
    }

    pub fn on_size_changed() {
    }

}


