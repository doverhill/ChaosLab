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

pub struct ConsoleServer {
    channel_handle: StormHandle,
    channel_address: *mut u8,
    on_get_capabilities: fn,
    on_set_text_color: fn,
    on_move_text_cursor: fn,
    on_draw_image_patch: fn,
    on_write_text: fn,
    on_write_objects: fn,
}

impl ConsoleServer {
    pub fn create() {
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


