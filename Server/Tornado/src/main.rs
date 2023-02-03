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
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = ServerState::new();

    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

    let mut tornado_server = TornadoServer::create(
        &mut process, 
        "Chaos", 
        "Tornado server", 
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    ).unwrap();

    loop {
        let event = StormProcess::wait_for_event().unwrap();
        println!("tornado: got event {:?}", event);
        console_client.process_event(&process, &event, &mut state);
        tornado_server.process_event(&process, &event, &mut state);
    }

    process.end();
}
