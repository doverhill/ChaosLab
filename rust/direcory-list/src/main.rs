extern crate chaos_core;

use chaos_core::{ process, service, handle::Handle };

fn main() {
    let result = service::connect("vfs", None, None, None);
    match result {
        Ok(service_handle) => {
            process::emit_debug(&format!("Connected to service handle: {}", service_handle));
            list(service_handle);
        },
        Err(error) => {
            process::emit_error(error, "Failed to connect to service");
        }
    }

    chaos_core::done();
}

fn list(service_handle: Handle) -> () {

}
