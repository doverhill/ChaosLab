extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;
use crate::types::*;

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// render(components: mixed list) -> _
// get_next() -> usize  // returns a counter local to each connection/client

pub trait BogusServerImplementation {
    fn simple_sum(&mut self, x: i32, y: i32) -> i32;
    fn get_files(&mut self, path: &str) -> Vec<FileInfo>;
    fn render(&mut self, components: crate::RenderHandleIterator);
    fn get_next(&mut self) -> usize;
}

lazy_static! {
    // Service handle -> BogusServer
    static ref SERVERS: Mutex<HashMap<Handle, Arc<Mutex<BogusServer>>>> = {
        Mutex::new(HashMap::new())
    };

    // Channel handle -> Service handle
    static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = {
        Mutex::new(HashMap::new())
    };

    // Channel handle -> BogusServerImplementation
    static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn BogusServerImplementation + Send>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct BogusServer {
    // service_reference: Arc<Mutex<Service>>,
    // initialized: bool,
    pub implementation_factory: fn() -> Box<dyn BogusServerImplementation + Send>
}

impl BogusServer {
    pub fn from_service(service_reference: Arc<Mutex<Service>>, implementation_factory: fn() -> Box<dyn BogusServerImplementation + Send>) -> Arc<Mutex<BogusServer>> {
        let server = BogusServer {
            // service_reference: service_reference.clone(),
            // initialized: false,
            implementation_factory: implementation_factory
        };

        let service = service_reference.lock().unwrap();

        // register this server as handler for this service
        let server_reference = Arc::new(Mutex::new(server));
        let mut servers = SERVERS.lock().unwrap();
        servers.insert(service.handle, server_reference.clone());

        server_reference
    }

    pub fn default(implementation_factory: fn() -> Box<dyn BogusServerImplementation + Send>) -> Result<Arc<Mutex<BogusServer>>, Error> {
        // create default service
        match Service::create("test", "Chaos", "Test server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()) {
            Ok(service_reference) => {
                let mut service = service_reference.lock().unwrap();
                service.on_connect(Self::handle_connect).unwrap();
                // let service_handle = service.handle;
                // drop(service);

                let server = BogusServer {
                    // service_reference: service_reference.clone(),
                    // initialized: false,
                    implementation_factory: implementation_factory
                };

                // register this server as handler for this service
                let server_reference = Arc::new(Mutex::new(server));
                let mut servers = SERVERS.lock().unwrap();
                servers.insert(service.handle, server_reference.clone());

                Ok(server_reference)
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to create service").unwrap();
                Err(error)
            }
        }
    }

    fn handle_connect(service_reference: &Arc<Mutex<Service>>, channel_reference: Arc<Mutex<Channel>>) {
        let service = service_reference.lock().unwrap();
        let servers = SERVERS.lock().unwrap();
        if let Some(server_reference) = servers.get(&service.handle) {
            let mut channels = CHANNELS.lock().unwrap();
            let mut channel = channel_reference.lock().unwrap();
            channels.insert(channel.handle, service.handle);
            channel.on_message(Self::handle_message).unwrap();

            let mut implementations = IMPLEMENTATIONS.lock().unwrap();
            let server = server_reference.lock().unwrap();
            let implementation = (server.implementation_factory)();
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
                crate::client_to_server_calls::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE => {
                    crate::client_to_server_calls::simple_sum_call::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_GET_FILES_CLIENT_MESSAGE => {
                    crate::client_to_server_calls::get_files_call::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_RENDER_CLIENT_MESSAGE => {
                    crate::client_to_server_calls::render_call::handle(implementation, channel_reference);
                },
                crate::client_to_server_calls::BOGUS_GET_NEXT_CLIENT_MESSAGE => {
                    crate::client_to_server_calls::get_next_call::handle(implementation, channel_reference);
                },
                _ => {
                    panic!("Unknown message {} for protocol Bogus", message);
                }
            }
        }
    }
}