#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;

use alloc::string::ToString;
use library_chaos::*;
use protocol_console::*;

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Cluido").unwrap();

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

    console_client.write_text(&WriteTextParameters { text: "Cluido".to_string() });

    loop {
        StormProcess::wait_for_event();
    }

    process.end();
}
