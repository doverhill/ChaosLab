extern crate chaos_core;
use chaos_core::process;

fn main() {
    process::emit_information("directory-list 0.1");
    
    // var channel = service::connect().unwrap();

    chaos_core::done();
}
