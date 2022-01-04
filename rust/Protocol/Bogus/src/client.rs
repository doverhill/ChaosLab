extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use library_chaos::{ Channel, Error, Process, Service };

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// render(components: mixed list) -> _
// get_next() -> usize  // returns a counter local to each connection/client

pub struct BogusClient {
    channel_reference: Arc<Mutex<Channel>>
}

impl BogusClient {
    pub fn from_channel(channel_reference: Arc<Mutex<Channel>>) -> Self {
        BogusClient {
            channel_reference: channel_reference
        }
    }

    pub fn default() -> Result<Self, Error> {
        // attempt to connect to the test service
        match Service::connect("test", None, None, None, 4096) {
            Ok(channel_reference) => {
                Process::emit_information("Connected to service").unwrap();
                let mut channel = channel_reference.lock().unwrap();
                channel.initialize("bogus", 1);
                drop(channel);

                Ok(BogusClient {
                    channel_reference: channel_reference
                })
            },
            Err(error) => {
                Process::emit_error(&error, "Failed to connect to VFS service").unwrap();
                Err(error)
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