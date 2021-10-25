extern crate chaos;

use chaos::{ process, service, channel::Channel };

fn main() {
    // process::wrap will not be needed when running natively on chaos
    process::wrap("Application.DirectoryList", chaos_entry);
}

fn chaos_entry() {
    match service::connect("vfs", None, None, None) {
        Ok(channel) => {
            list(channel);
        },
        Err(error) => {
            process::emit_error(error, "Failed to connect to service");
        }
    }
}

fn list(channel: Channel) -> () {
    unsafe {
        *channel.map_pointer = 42;
    }

    channel.interface(1)
        .then(|pointer: *mut u8| {
            unsafe {
                let r = *pointer;
                process::emit_information(&format!("Got result: {}", r));
            }
            process::end();
        })
        .orelse(|error| {
            process::emit_error(error, "Call failed");
            process::end();
        })
        .call();

    process::run();
}
