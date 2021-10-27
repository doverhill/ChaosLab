extern crate chaos;

use chaos::{ process, service, handle::Handle };

fn main() {
    // process::wrap will not be needed when running natively on chaos
    process::wrap("Application.DirectoryList", chaos_entry);
}

fn chaos_entry() {
    match service::connect("vfs", None, None, None) {
        Ok(channel_handle) => {
            list(channel_handle);
        },
        Err(error) => {
            process::emit_error(error, "Failed to connect to service");
        }
    }
}

fn list(channel_handle: Handle) -> () {
    unsafe {
        let pointer = process::get_channel_pointer(channel_handle).unwrap();
        *pointer = 42;
    }

    process::channel_interface(channel_handle, 1)
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
