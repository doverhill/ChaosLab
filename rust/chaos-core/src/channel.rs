use crate::handle::Handle;
use std::fmt;
use shared_memory::*;
use std::sync::Mutex;
use std::collections::HashMap;

pub struct Channel {
    handle: Handle,
    buffer: *mut u8
}

impl Channel {
    pub fn new(channel_handle: Handle) -> Channel {
        let memory_name = &Channel::get_name(&channel_handle);

        let shared_memory = match ShmemConf::new().size(4096).os_id(memory_name).create() {
            Ok(memory) => memory,
            Err(ShmemError::MappingIdExists) => {
                println!("Could not create, opening instead");
                ShmemConf::new().os_id(memory_name).open().unwrap()
            },
            Err(e) => {
                println!("Got error {}", e);
                panic!("Could not create shared memory");
            }
        };

        println!("Got shared memory @ {:p}", shared_memory.as_ptr());

        Channel {
            handle: channel_handle,
            buffer: shared_memory.as_ptr() as *mut u8
        }
    }

    pub fn get_name(channel_handle: &Handle) -> String {
        return format!("__chaos_channel_{}", channel_handle.id);
    }

    pub fn write(&self, value: u8) -> () {
        unsafe {
            println!("reading");
            let v = std::ptr::read_volatile(self.buffer);
            println!("read {}", v);
            std::ptr::write_volatile(self.buffer, value);
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[CHANNEL: handle: {}, buffer: {:p}]", self.handle, self.buffer)
    }
}

lazy_static! {
    static ref ON_MESSAGE: Mutex<HashMap<u64, fn(Channel) -> ()>> = {
        Mutex::new(HashMap::new())
    };
}

pub fn on_message(channel: Channel, handler: Option<fn(Channel) -> ()>) {
    match handler {
        Some(f) => {
            ON_MESSAGE.lock().unwrap().insert(channel.handle.id, f);
        },
        None => {
            ON_MESSAGE.lock().unwrap().remove(&channel.handle.id);
        }
    }
}
