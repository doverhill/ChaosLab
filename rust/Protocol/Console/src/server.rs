extern crate library_chaos;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<ConsoleServer>>>> = {
        Mutex::new(HashMap::new())
    };
    static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = {
        Mutex::new(HashMap::new())
    };
    static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn ConsoleServerImplementation + Send>>> = {
        Mutex::new(HashMap::new())
    };
}

pub trait ConsoleServerImplementation {
    fn get_capabilities(&mut self) -> (bool, usize, usize, usize, usize);
    fn set_text_color(&mut self, color: Color, background_color: Color);
    fn set_text_cursor_position(&mut self, column: usize, row: usize);
    fn write_text(&mut self, text: &str);
    fn render_bitmap_patches(&mut self, objects: crate::RenderBitmapPatchesBitmapPatchIterator);
}

pub struct ConsoleServer {
    pub implementation_factory: fn() -> Box<dyn ConsoleServerImplementation + Send>
}

impl ConsoleServer {
    pub fn from_service(service_reference: Arc<Mutex<Service>>, implementation_factory: fn() -> Box<dyn ConsoleServerImplementation + Send>) -> Arc<Mutex<ConsoleServer>> {
        let instance = ConsoleServer {
            implementation_factory: implementation_factory
        };

        let mut service = service_reference.lock().unwrap();
        service.on_connect(Self::handle_connect).unwrap();

        let instance_reference = Arc::new(Mutex::new(instance));
        let mut instances = INSTANCES.lock().unwrap();
        instances.insert(service.handle, instance_reference.clone());

        instance_reference
    }

    pub fn default(vendor: &str, description: &str, implementation_factory: fn() -> Box<dyn ConsoleServerImplementation + Send>) -> Result<Arc<Mutex<ConsoleServer>>, Error> {
        match Service::create("Console", vendor, description, Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()) {
            Ok(service_reference) => {
                Ok(Self::from_service(service_reference, implementation_factory))
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to create service").unwrap();
                Err(error)
            }
        }
    }

    fn handle_connect(service_reference: &Arc<Mutex<Service>>, channel_reference: Arc<Mutex<Channel>>) {
        let service = service_reference.lock().unwrap();
        let instances = INSTANCES.lock().unwrap();
        if let Some(instance_reference) = instances.get(&service.handle) {
            let mut channels = CHANNELS.lock().unwrap();
            let mut channel = channel_reference.lock().unwrap();
            channels.insert(channel.handle, service.handle);
            channel.on_message(Self::handle_message).unwrap();
            let mut implementations = IMPLEMENTATIONS.lock().unwrap();
            let instance = instance_reference.lock().unwrap();
            let implementation = (instance.implementation_factory)();
            implementations.insert(channel.handle, implementation);
        }
    }

    fn handle_message(channel_reference: Arc<Mutex<Channel>>, message: u64) {
        let channel = channel_reference.lock().unwrap();
        let channel_handle = channel.handle;
        drop(channel);

        let mut implementations = IMPLEMENTATIONS.lock().unwrap();
        if let Some(implementation) = implementations.get_mut(&channel_handle) {
            match message {
                crate::client_to_server_calls::CONSOLE_GET_CAPABILITIES_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::get_capabilities::handle(implementation, channel_reference);
                }
                crate::client_to_server_calls::CONSOLE_SET_TEXT_COLOR_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::set_text_color::handle(implementation, channel_reference);
                }
                crate::client_to_server_calls::CONSOLE_SET_TEXT_CURSOR_POSITION_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::set_text_cursor_position::handle(implementation, channel_reference);
                }
                crate::client_to_server_calls::CONSOLE_WRITE_TEXT_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::write_text::handle(implementation, channel_reference);
                }
                crate::client_to_server_calls::CONSOLE_RENDER_BITMAP_PATCHES_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::render_bitmap_patches::handle(implementation, channel_reference);
                }
                _ => {
                    panic!("Unknown message {} received for protocol Console", message);
                }
            }
        }
    }

    pub fn key_pressed(&self, key_code: usize) -> Result<(), Error> {
        crate::server_to_client_calls::key_pressed::call(self.channel_reference.clone(), key_code)
    }

    pub fn key_released(&self, key_code: usize) -> Result<(), Error> {
        crate::server_to_client_calls::key_released::call(self.channel_reference.clone(), key_code)
    }

    pub fn text_available(&self, text: &str) -> Result<(), Error> {
        crate::server_to_client_calls::text_available::call(self.channel_reference.clone(), text)
    }

    pub fn pointer_moved(&self, x: usize, y: usize) -> Result<(), Error> {
        crate::server_to_client_calls::pointer_moved::call(self.channel_reference.clone(), x, y)
    }

    pub fn pointer_button_pressed(&self, x: usize, y: usize, button_number: usize) -> Result<(), Error> {
        crate::server_to_client_calls::pointer_button_pressed::call(self.channel_reference.clone(), x, y, button_number)
    }

    pub fn pointer_button_released(&self, x: usize, y: usize, button_number: usize) -> Result<(), Error> {
        crate::server_to_client_calls::pointer_button_released::call(self.channel_reference.clone(), x, y, button_number)
    }
}
