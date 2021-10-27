extern crate chaos;
use chaos::{ process, service, handle::Handle, channel::Channel };
use uuid::Uuid;

fn main() {
    // process::wrap will not be needed when running natively on chaos
    process::wrap("Server.VFS", chaos_entry);
}

fn chaos_entry() {
    process::emit_information("Starting VFS server");

    let result = service::create("vfs", "Chaos", "Virtual file system server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap());
    match result {
        Ok(service_handle) => {
            process::emit_debug(&format!("Created service handle {}", service_handle));
            handle_service(service_handle);
        },
        Err(error) => {
            process::emit_error(error, "Failed to create service");
        }
    }
}

fn handle_channel_signal(channel: &Channel, signal: u64) -> () {
    unsafe {
        let v = *channel.map_pointer;
        *channel.map_pointer = v + 4;
    }
    process::channel_interface(channel.channel_handle, 1).call();
}

fn handle_connect(service_handle: Handle, channel: &Channel) -> () {
    process::emit_debug(&format!("Connect on service handle {}, got channel {}", service_handle, channel));
    process::on_signal(channel, Some(handle_channel_signal));
}

fn handle_service(service_handle: Handle) -> () {
    process::on_connect(service_handle, Some(handle_connect));
    let error = process::run();
    process::emit_error(error, "Event loop error");
}