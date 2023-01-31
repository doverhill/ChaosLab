// #![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::GlobalState;

use library_chaos::{StormEvent, ChannelHandle, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = GlobalState::new();

    // connect to console
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

    console_client.write_text(&WriteTextParameters { text: "Hello from Tornado!".to_string() });

    console_client.on_pointer_moved(|_| {
        println!("tornado: pointer moved");
    });

    // set up service
    let mut tornado_server = TornadoServer::create(&mut process, "Chaos", "Tornado server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    tornado_server.on_client_connected(|channel_handle| {
        println!("tornado: client connected");
        state.add_client(channel_handle);
    });

    tornado_server.on_client_disconnected(|channel_handle| {
        println!("tornado: client disconnected");
        state.remove_client(channel_handle);
    });

    // run
    process.run();
}

