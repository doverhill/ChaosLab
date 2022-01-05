extern crate library_chaos;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<ConsoleClient>>>> = {
        Mutex::new(HashMap::new())
    };
    static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = {
        Mutex::new(HashMap::new())
    };
    static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn ConsoleClientImplementation + Send>>> = {
        Mutex::new(HashMap::new())
    };
}

pub trait ConsoleClientImplementation {
    fn key_pressed(&mut self, key_code: usize);
    fn key_released(&mut self, key_code: usize);
    fn text_available(&mut self, text: &str);
    fn pointer_moved(&mut self, x: usize, y: usize);
    fn pointer_button_pressed(&mut self, x: usize, y: usize, button_number: usize);
    fn pointer_button_released(&mut self, x: usize, y: usize, button_number: usize);
}

pub struct ConsoleClient {
    channel_reference: Arc<Mutex<Channel>>,
    pub implementation_factory: fn() -> Box<dyn ConsoleClientImplementation + Send>
}

impl ConsoleClient {
    pub fn from_channel(channel_reference: Arc<Mutex<Channel>>) -> Self
    fn handle_message(channel_reference: Arc<Mutex<Channel>>, message: u64) {
        let channel = channel_reference.lock().unwrap();
        let channel_handle = channel.handle;
        drop(channel);

        let mut implementations = IMPLEMENTATIONS.lock().unwrap();
        if let Some(implementation) = implementations.get_mut(&channel_handle) {
            match message {
                crate::server_to_client_calls::CONSOLE_KEY_PRESSED_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::key_pressed::handle(implementation, channel_reference);
                }
                crate::server_to_client_calls::CONSOLE_KEY_RELEASED_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::key_released::handle(implementation, channel_reference);
                }
                crate::server_to_client_calls::CONSOLE_TEXT_AVAILABLE_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::text_available::handle(implementation, channel_reference);
                }
                crate::server_to_client_calls::CONSOLE_POINTER_MOVED_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::pointer_moved::handle(implementation, channel_reference);
                }
                crate::server_to_client_calls::CONSOLE_POINTER_BUTTON_PRESSED_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::pointer_button_pressed::handle(implementation, channel_reference);
                }
                crate::server_to_client_calls::CONSOLE_POINTER_BUTTON_RELEASED_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::pointer_button_released::handle(implementation, channel_reference);
                }
                _ => {
                    panic!("Unknown message {} received for protocol Console", message);
                }
            }
        }
    }

    pub fn get_capabilities(&self) -> Result<(bool, usize, usize, usize, usize), Error> {
        crate::client_to_server_calls::get_capabilities::call(self.channel_reference.clone())
    }

    pub fn set_text_color(&self, color: Color, background_color: Color) -> Result<(), Error> {
        crate::client_to_server_calls::set_text_color::call(self.channel_reference.clone(), color, background_color)
    }

    pub fn set_text_cursor_position(&self, column: usize, row: usize) -> Result<(), Error> {
        crate::client_to_server_calls::set_text_cursor_position::call(self.channel_reference.clone(), column, row)
    }

    pub fn write_text(&self, text: &str) -> Result<(), Error> {
        crate::client_to_server_calls::write_text::call(self.channel_reference.clone(), text)
    }

    pub fn render_bitmap_patches(&self, objects: Vec<crate::BitmapPatch>) -> Result<(), Error> {
        crate::client_to_server_calls::render_bitmap_patches::call(self.channel_reference.clone(), objects)
    }
}
