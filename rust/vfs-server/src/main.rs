extern crate chaos_core;
#[macro_use]

use chaos_core::{ process, service, handle::Handle };
use uuid::Uuid;

fn main() {
    process::emit_information("Starting VFS server");

    let result = service::create("vfs", "Chaos", "Virtual file system server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap());
    match result {
        Ok(service_handle) => {
            process::emit_debug(&format!("Created service handle: {}", service_handle));
            handle_service(service_handle);
        },
        Err(error) => {
            process::emit_error(error, "Failed to create service");
        }
    }

    chaos_core::done();
}

fn handle_connect() -> () {
    process::emit_debug("Connect on service");
}

fn handle_service(service_handle: Handle) -> () {
    service_handle.on_connect = handle_connect;

    process::run();
}