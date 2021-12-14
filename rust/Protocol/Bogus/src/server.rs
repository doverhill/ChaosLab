extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// fib(n: usize) -> [fib numbers]
// render(mixed list) -> _

lazy_static! {
    static ref SERVERS: Mutex<HashMap<Handle, Arc<Mutex<Service>>>> = {
        Mutex::new(HashMap::new())
    };

    static ref CHANNELS: Mutex<HashMap<Handle, Arc<Mutex<Channel>>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct BogusServer {
    service_reference: Arc<Mutex<Service>>,
    initialized: bool,
    simple_sum_handler: Option<fn(i32, i32) -> i32>
}

impl BogusServer {
    pub fn from_service(service_reference: Arc<Mutex<Service>>) -> Self {
        BogusServer {
            service_reference: service_reference,
            initialized: false,
            simple_sum_handler: None
        }
    }

    pub fn default() -> Result<Self, Error> {
        // create default service
        match Service::create("test", "Chaos", "Test server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()) {
            Ok(service_reference) => {
                

                let mut service = service_reference.lock().unwrap();
                service.on_connect(Self::handle_connect).unwrap();
                drop(service);

                Ok(BogusServer {
                    service_reference: service_reference,
                    initialized: false,
                    simple_sum_handler: None
                })
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to create service").unwrap();
                Err(error)
            }
        }
    }

    fn handle_connect(service_reference: &Arc<Mutex<Service>>, channel_reference: Arc<Mutex<Channel>>) {
        let mut channel = channel_reference.lock().unwrap();
        channel.on_message(Self::handle_message).unwrap();
    }
    
    fn handle_message(channel_reference: &Arc<Mutex<Channel>>, message: u64) {
        println!("message on channel");

        let channel = channel_reference.lock().unwrap();
        match message {
            crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE => {
                if let Some(handler) = self.simple_sum_handler {

                }
            },
            _ => {
                panic!("Unknown message {} for protocol Bogus", message);
            }
        }
    }

    pub fn on_simple_sum(&mut self, handler: fn(i32, i32) -> i32) {
        self.ensure_initialized();
        self.simple_sum_handler = Some(handler);
    }
}