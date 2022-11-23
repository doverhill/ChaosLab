extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use std::iter::Iterator;
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;
use crate::types::*;

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// fib(n: usize) -> [fib numbers]
// render(components: mixed list) -> _
// get_next() -> usize  // returns a counter local to each connection/client

pub struct Window {
    
}

pub struct Button {

}

pub enum Component {
    Window(Window),
    Button(Button)
}

pub trait BogusServerImplementation {
    fn simple_sum(&mut self, x: i32, y: i32) -> i32;
    fn get_files(&mut self, path: &str) -> Vec<FileInfo>;
    fn fib(&mut self, n: usize) -> Vec<usize>;
    fn render(&mut self, components: &dyn Iterator<Item = Component>);
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
    service_reference: Arc<Mutex<Service>>,
    initialized: bool,
    pub implementation_factory: fn() -> Box<dyn BogusServerImplementation + Send>
}

impl BogusServer {
    pub fn from_service(service_reference: Arc<Mutex<Service>>, implementation_factory: fn() -> Box<dyn BogusServerImplementation + Send>) -> Arc<Mutex<BogusServer>> {
        let server = BogusServer {
            service_reference: service_reference.clone(),
            initialized: false,
            implementation_factory: implementation_factory
        };

        let service = service_reference.lock().unwrap();

        // register this server as handler for this service
        let server_reference = Arc::new(Mutex::new(server));
        let servers = &mut *SERVERS.lock().unwrap();
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
                    service_reference: service_reference.clone(),
                    initialized: false,
                    implementation_factory: implementation_factory
                };

                // register this server as handler for this service
                let server_reference = Arc::new(Mutex::new(server));
                let servers = &mut *SERVERS.lock().unwrap();
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
        let servers = &*SERVERS.lock().unwrap();
        if let Some(server_reference) = servers.get(&service.handle) {
            let channels = &mut *CHANNELS.lock().unwrap();
            let mut channel = channel_reference.lock().unwrap();
            channels.insert(channel.handle, service.handle);
            channel.on_message(Self::handle_message).unwrap();

            let implementations = &mut *IMPLEMENTATIONS.lock().unwrap();
            let server = &*server_reference.lock().unwrap();
            let implementation = (server.implementation_factory)();
            implementations.insert(channel.handle, implementation);
        }
    }
    
    fn handle_message(channel_reference: &Arc<Mutex<Channel>>, message: u64) {
        let channel = &mut *channel_reference.lock().unwrap();
        let implementations = &mut *IMPLEMENTATIONS.lock().unwrap();
        if let Some(implementation) = implementations.get_mut(&channel.handle) {
            match message {
                crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE => {
                    crate::simple_sum_call::handle(implementation, channel);
                },
                crate::client::BOGUS_GET_FILES_CLIENT_MESSAGE => {
                    crate::get_files_call::handle(implementation, channel);
                },
                _ => {
                    panic!("Unknown message {} for protocol Bogus", message);
                }
            }
        }
    }
}