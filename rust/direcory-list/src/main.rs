extern crate chaos_core;

use chaos_core::{ process, service, channel::Channel };

fn main() {
    process::set_info("DirectoryList (rust)");

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
    unsafe {
        while std::ptr::read_volatile(channel.map_pointer) == 0 {}
        let value = std::ptr::read_volatile(channel.map_pointer);
        process::emit_information(&format!("Got {} from server", value));
    }
}
