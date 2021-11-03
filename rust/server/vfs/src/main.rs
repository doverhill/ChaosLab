extern crate chaos;

use chaos::{ process::Process, service::Service, channel::Channel };
use std::sync::Arc;
use uuid::Uuid;
use std::sync::Mutex;

#[derive(Clone, Copy)]
struct ChannelCall {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
struct ChannelResponse {
    result: i32,
    diff: i32,
}

fn main() {
    // to be nice, lets set a name for our application
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
    Process::emit_debug(&format!("Message {} on {}", message, channel));

    let data = channel.get::<ChannelCall>();
    let result = ChannelResponse {
        result: data.x + data.y,
        diff: data.x - data.y
    };
    channel.set(result);
    channel.send(1);
}
