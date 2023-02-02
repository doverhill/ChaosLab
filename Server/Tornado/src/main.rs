// #![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::ServerState;

use library_chaos::{StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::<ServerState, ServerState>::new("Server.Tornado").unwrap();
    let state = ServerState::new();

    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();
    console_client.attach_observer(&state);

    let mut tornado_server = TornadoServer::create(
        &mut process, 
        "Chaos", 
        "Tornado server", 
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    ).unwrap();
    tornado_server.attach_observer(&state);

    process.run();
}
