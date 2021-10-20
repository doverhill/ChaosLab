extern crate chaos_core;

use chaos_core::{ process, service, channel::Channel };

fn main() {
    process::set_info("DirectoryList");

    let result = service::connect("vfs", None, None, None);
    match result {
        Ok(channel) => {
            process::emit_debug(&format!("Connected to service, got channel {}", channel));
            list(channel);
        },
        Err(error) => {
            process::emit_error(error, "Failed to connect to service");
        }
    }

    chaos_core::done();
}

fn list(channel: Channel) -> () {
    channel.write(42);
}
