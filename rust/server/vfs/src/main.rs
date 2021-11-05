extern crate chaos;

use chaos::{ process::Process, service::Service, channel::Channel };
use std::sync::Arc;
use uuid::Uuid;
use std::sync::Mutex;

struct ChannelCall {
    msg: [u8; 100],
    x: i32
}

impl ChannelCall {
    pub fn new(msg: &str, x: i32) -> ChannelCall {
        let mut tmp = ChannelCall {
            msg: [0u8; 100],
            x: x
        };
        unsafe {
            core::ptr::copy(msg.as_ptr(), core::ptr::addr_of!(tmp.msg) as *mut u8, core::cmp::min(99, msg.len()));
        }
        tmp
    }

    pub fn get_msg(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.msg) }
    }
}

struct ChannelResponse {
    length: i32,
    new_msg: [u8; 100]
}

impl ChannelResponse {
    pub fn new(length: i32, new_msg: &str) -> ChannelResponse {
        let mut tmp = ChannelResponse {
            length: length,
            new_msg: [0u8; 100]
        };
        unsafe {
            core::ptr::copy(new_msg.as_ptr(), core::ptr::addr_of!(tmp.new_msg) as *mut u8, core::cmp::min(99, new_msg.len()));
        }
        tmp
    }

    pub fn get_new_msg(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.new_msg) }
    }
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Server.VFS");
    Process::emit_debug("Starting VFS server");

    match Service::create("vfs", "Chaos", "Virtual file system server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()) {
        Ok(service_wrap) => {
            {
                let mut service = service_wrap.lock().unwrap();
                Process::emit_debug(&format!("Created service {}", service));
                service.on_connect(handle_connect);
            }
            let error = Process::run();
            Process::emit_error(error, "Event loop error");
        }
        Err(error) => {
            Process::emit_error(error, "Failed to create service");
        }
    }

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}

fn handle_connect(service_wrap: &Arc<Mutex<Service>>, channel_wrap: Arc<Mutex<Channel>>) {
    let service = service_wrap.lock().unwrap();
    let mut channel = channel_wrap.lock().unwrap();
    Process::emit_debug(&format!("Connect on {} -> {}", service, channel));
    channel.on_message(handle_message);
}

fn handle_message(channel_wrap: &Arc<Mutex<Channel>>, message: u64) {
    let channel = channel_wrap.lock().unwrap();
    let data = channel.get::<ChannelCall>();
    channel.set(ChannelResponse::new(data.msg.len() as i32, "hej"));
    channel.send(1);
}
