#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::GlobalState;

use library_chaos::{StormEvent, StormHandle, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = GlobalState::new();

    // connect to console
    let console_client = ConsoleClient::connect_first(&mut process).unwrap();

    console_client.write_text("Hello from Tornado!");

    // set up service
    let tornado_server = TornadoServer::create(&mut process, "Chaos", "Tornado server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    // set up logic
    console_client.on_pointer_moved(|| {
        // println!("pointer moved");
    });



    tornado_server.on_connect(|channel_handle| {
        state.add_client(channel_handle);
    });

    tornado_server.on_disconnect(|channel_handle| {
        state.remove_client(channel_handle);
    });

    // run
    process.run();
}

