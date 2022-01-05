extern crate library_chaos;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use library_chaos::{ Channel, Error, Process, Service, Handle };
use uuid::Uuid;

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<BogusAutoClient>>>> = {
        Mutex::new(HashMap::new())
    };
    static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = {
        Mutex::new(HashMap::new())
    };
    static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn BogusAutoClientImplementation + Send>>> = {
        Mutex::new(HashMap::new())
    };
}

pub trait BogusAutoClientImplementation {
}

pub struct BogusAutoClient {
    channel_reference: Arc<Mutex<Channel>>
}

impl BogusAutoClient {
    pub fn from_channel(channel_reference: Arc<Mutex<Channel>>) -> Self {
        BogusAutoClient {
            channel_reference: channel_reference
        }
    }

    pub fn default() -> Result<Self, Error> {
        match Service::connect("BogusAuto", None, None, None, 4096) {
            Ok(channel_reference) => {
                let mut channel = channel_reference.lock().unwrap();
                channel.initialize("BogusAuto", 1);
                drop(channel);

                Ok(BogusAutoClient {
                    channel_reference: channel_reference
                })
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to connect to BogusAuto service").unwrap();
                Err(error)
            }
        }
    }

    fn handle_message(channel_reference: Arc<Mutex<Channel>>, message: u64) {
        let channel = channel_reference.lock().unwrap();
        let channel_handle = channel.handle;
        drop(channel);

        let mut implementations = IMPLEMENTATIONS.lock().unwrap();
        if let Some(implementation) = implementations.get_mut(&channel_handle) {
            match message {
                _ => {
                    panic!("Unknown message {} received for protocol BogusAuto", message);
                }
            }
        }
    }

    pub fn simple_sum(&self, x: i32, y: i32) -> Result<i32, Error> {
        crate::client_to_server_calls::simple_sum::call(self.channel_reference.clone(), x, y)
    }

    pub fn get_files(&self, path: &str) -> Result<crate::GetFilesFileInfoIterator, Error> {
        crate::client_to_server_calls::get_files::call(self.channel_reference.clone(), path)
    }

    pub fn render(&self, objects: Vec<crate::RenderArgumentsEnum>) -> Result<(), Error> {
        crate::client_to_server_calls::render::call(self.channel_reference.clone(), objects)
    }

    pub fn get_next(&self) -> Result<usize, Error> {
        crate::client_to_server_calls::get_next::call(self.channel_reference.clone())
    }

    pub fn both_mixed(&self, objects: Vec<crate::BothMixedArgumentsEnum>) -> Result<crate::BothMixedMixedResultIterator, Error> {
        crate::client_to_server_calls::both_mixed::call(self.channel_reference.clone(), objects)
    }
}
