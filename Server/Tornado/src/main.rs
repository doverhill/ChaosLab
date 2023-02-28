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
    // set up process and server state
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = ServerState::new();

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();
    console_client.write_text(&WriteTextParameters { text: "hello console".to_string() });

    // create tornado service
    let mut tornado_server = TornadoServer::create(
        &mut process, 
        "Chaos", 
        "Tornado server", 
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    ).unwrap();

    // main event loop
    loop {
        let event = StormProcess::wait_for_event().unwrap();
        console_client.process_event(&process, &event, &mut state);
        tornado_server.process_event(&mut process, &event, &mut state);
    }

    process.end();
}
