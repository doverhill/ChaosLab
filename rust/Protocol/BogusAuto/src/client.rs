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
    pub implementation_factory: fn() -> Box<dyn BogusAutoClientImplementation + Send>
}

impl BogusAutoClient {
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

    pub fn simple_sum() {}

    pub fn get_files() {}

    pub fn render() {}

    pub fn get_next() {}

    pub fn both_mixed() {}
}
