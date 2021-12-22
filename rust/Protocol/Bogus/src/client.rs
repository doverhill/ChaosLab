extern crate library_chaos;

use std::sync::{ Arc, Mutex };
use library_chaos::{ Channel, Error, Process, Service };

// calls
// simple_sum(x: i32, y: i32) -> i32
// get_files(path: &str) -> [file: FileInfo]
// fib(n: usize) -> [fib numbers]
// render(mixed list) -> _

pub const BOGUS_SIMPLE_SUM_CLIENT_MESSAGE: u64 = 1;
pub const BOGUS_GET_FILES_CLIENT_MESSAGE: u64 = 2;
pub const BOGUS_FIB_CLIENT_MESSAGE: u64 = 3;
pub const BOGUS_RENDER_CLIENT_MESSAGE: u64 = 4;

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
        crate::simple_sum_call::call(self.channel_reference.clone(), x, y)
    }

    pub fn get_files(&self, path: &str) -> Result<crate::get_files_call::ReturnIterator, Error> {
        crate::get_files_call::call(self.channel_reference.clone(), path)
    }
}