extern crate library_chaos;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<BogusAutoServer>>>> = {
        Mutex::new(HashMap::new())
    };
    static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = {
        Mutex::new(HashMap::new())
    };
    static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn BogusAutoServerImplementation + Send>>> = {
        Mutex::new(HashMap::new())
    };
}

pub trait BogusAutoServerImplementation {
    fn simple_sum(&mut self, x: i32, y: i32) -> i32;
    fn get_files(&mut self, path: &str) -> Vec<crate::FileInfo>;
    fn render(&mut self, objects: crate::RenderMixedArgumentsIterator);
    fn get_next(&mut self) -> usize;
    fn both_mixed(&mut self, objects: crate::BothMixedMixedArgumentsIterator) -> Vec<crate::BothMixedResultEnum>;
}

pub struct BogusAutoServer {
    pub implementation_factory: fn() -> Box<dyn BogusAutoServerImplementation + Send>
}

impl BogusAutoServer {
    pub fn from_service(service_reference: Arc<Mutex<Service>>, implementation_factory: fn() -> Box<dyn BogusAutoServerImplementation + Send>) -> Arc<Mutex<Self>> {
        let instance = BogusAutoServer {
            implementation_factory: implementation_factory
        };

        let instance_reference = Arc::new(Mutex::new(instance));
        let mut instances = INSTANCES.lock().unwrap();
        let mut service = service_reference.lock().unwrap();
        instances.insert(service.handle, instance_reference.clone());
        service.on_connect(Self::handle_connect).unwrap();

        instance_reference
    }

    pub fn default(vendor: &str, description: &str, implementation_factory: fn() -> Box<dyn BogusAutoServerImplementation + Send>) -> Result<Arc<Mutex<Self>>, Error> {
        match Service::create("BogusAuto", vendor, description, Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()) {
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
                crate::client_to_server_calls::BOGUS_AUTO_SIMPLE_SUM_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::simple_sum::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_AUTO_GET_FILES_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::get_files::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::render::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_AUTO_GET_NEXT_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::get_next::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_AUTO_BOTH_MIXED_CLIENT_TO_SERVER_MESSAGE => {
                    crate::client_to_server_calls::both_mixed::handle(implementation, channel_reference);
                },
                _ => {
                    panic!("Unknown client to server message {} received for protocol BogusAuto", message);
                }
            }
        }
    }

    pub fn notify(&self, channel_reference: Arc<Mutex<Channel>>, message: &str) -> Result<(), Error> {
        crate::server_to_client_calls::notify::call(channel_reference.clone(), message)
    }
}
