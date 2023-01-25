extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use library_chaos::{ Channel, Error, Process, Service };

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// render(components: mixed list) -> _
// get_next() -> usize  // returns a counter local to each connection/client

lazy_static! {
    // Channel handle -> BogusClient
    static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<BogusClient>>>> = {
        Mutex::new(HashMap::new())
    };
}

pub trait BogusClientImplementation {
    fn test(x: u32) -> u32;
}

pub struct BogusClient {
    channel_reference: Arc<Mutex<Channel>>,
    implementation: Box<dyn BogusClientImplementation + Send> 
}

impl BogusClient {
    pub fn from_channel(channel_reference: Arc<Mutex<Channel>>, implementation: Box<dyn BogusClientImplementation>) -> Arc<Mutex<Self>> {
        let instance = BogusClient {
            channel_reference: channel_reference,
            implementation: implementation
        };

        let mut channel = channel_reference.lock().unwrap();
        channel.initialize("bogus", 1);

        let instance_reference = Arc::new(Mutex::new(instance));
        let mut instances = INSTANCES.lock().unwrap();
        instances.insert(channel.handle, instance_reference.clone());

        channel.on_message(Self::handle_message).unwrap();

        instance_reference
    }

    pub fn default(implementation: Box<dyn BogusClientImplementation>) -> Result<Arc<Mutex<Self>>, Error> {
        // attempt to connect to the test service
        match Service::connect("test", None, None, None, 4096) {
            Ok(channel_reference) => {
                Ok(Self::from_channel(channel_reference, implementation))
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to connect to VFS service").unwrap();
                Err(error)
            }
        }
    }

    fn handle_message(channel_reference: Arc<Mutex<Channel>>, message: u64) {
        let channel = channel_reference.lock().unwrap();
        let channel_handle = channel.handle;
        drop(channel);

        let mut instances = INSTANCES.lock().unwrap();
        if let Some(instance) = instances.get_mut(&channel_handle) {
            match message {
                crate::server_to_client_calls::BOGUS_NOTIFY_CLIENT_MESSAGE => {
                    crate::server_to_client_calls::notify::handle(instance.implementation, channel_reference);
                },
                _ => {
                    panic!("Unknown message {} for protocol Bogus", message);
                }
            }
        }
    }

    pub fn simple_sum(&self, x: i32, y: i32) -> Result<i32, Error> {
        crate::client_to_server_calls::simple_sum_call::call(self.channel_reference.clone(), x, y)
    }

    pub fn get_files(&self, path: &str) -> Result<crate::client_to_server_calls::get_files_call::GetFilesCallIterator, Error> {
        crate::client_to_server_calls::get_files_call::call(self.channel_reference.clone(), path)
    }

    pub fn render_start(&self) {
        crate::client_to_server_calls::render_call::start(self.channel_reference.clone());
    }

    pub fn render_add(&self, component: crate::client_to_server_calls::render_call::RenderTypeArguments) {
        crate::client_to_server_calls::render_call::add(self.channel_reference.clone(), component);
    }

    pub fn render_done(&self) {
        crate::client_to_server_calls::render_call::call(self.channel_reference.clone());
    }

    pub fn get_next(&self) -> Result<usize, Error> {
        crate::client_to_server_calls::get_next_call::call(self.channel_reference.clone())
    }
}