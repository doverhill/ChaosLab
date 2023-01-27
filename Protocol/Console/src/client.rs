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

pub struct ConsoleClient {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_key_pressed: fn,
    on_key_released: fn,
    on_pointer_moved: fn,
    on_pointer_pressed: fn,
    on_pointer_released: fn,
    on_size_changed: fn,
}

impl ConsoleClient {
    pub fn create() {
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


