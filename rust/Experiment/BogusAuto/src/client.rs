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
}

pub trait BogusAutoClientImplementation {
    fn notify(&mut self, message: &str);
}

pub struct BogusAutoClient {
    channel_reference: Arc<Mutex<Channel>>,
    pub implementation: Box<dyn BogusAutoClientImplementation + Send>
}

impl BogusAutoClient {
    pub fn from_channel(channel_reference: Arc<Mutex<Channel>>, implementation: Box<dyn BogusAutoClientImplementation + Send>) -> Arc<Mutex<Self>> {
        let instance = BogusAutoClient {
            channel_reference: channel_reference.clone(),
            implementation: implementation
        };

        let mut channel = channel_reference.lock().unwrap();
        channel.initialize("BogusAuto", 1);

        let instance_reference = Arc::new(Mutex::new(instance));
        let mut instances = INSTANCES.lock().unwrap();
        instances.insert(channel.handle, instance_reference.clone());

        channel.on_message(Self::handle_message).unwrap();

        instance_reference
    }

    pub fn default(implementation: Box<dyn BogusAutoClientImplementation + Send>) -> Result<Arc<Mutex<Self>>, Error> {
        match Service::connect("BogusAuto", None, None, None, 4096) {
            Ok(channel_reference) => {
                Ok(Self::from_channel(channel_reference, implementation))
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

        let instances = INSTANCES.lock().unwrap();
        if let Some(instance_reference) = instances.get(&channel_handle) {
            let mut instance = instance_reference.lock().unwrap();
            match message {
                crate::server_to_client_calls::BOGUS_AUTO_NOTIFY_SERVER_TO_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::notify::handle(&mut instance.implementation, channel_reference);
                },
                _ => {
                    panic!("Unknown server to client message {} received for protocol BogusAuto", message);
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
