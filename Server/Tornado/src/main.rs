extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

use library_chaos::{StormEvent, StormHandle, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

struct State {

}

fn main() {
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = State {};

    // connect to console
    let console_channel_handle = process
        .services
        .connect(CONSOLE_PROTOCOL_NAME)
        .unwrap();

    let console = ConsoleChannel::new(console_channel_handle, false);
    console.on_pointer_moved = |parameters| handle_pointer_moved(parameters);

    // set up service
    let tornado_service = process
        .services
        .create(
            TORNADO_PROTOCOL_NAME,
            "Chaos",
            "Tornado server",
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        )
        .unwrap();

    

    process.run();
}

fn handle_pointer_moved(parameters: PointerMovedParameters) {
    println!("pointer moved");
}