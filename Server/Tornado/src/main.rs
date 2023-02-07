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

    console_client.write_text(&WriteTextParameters { text: "hello console".to_string() });

    let mut tornado_server = TornadoServer::create(
        &mut process, 
        "Chaos", 
        "Tornado server", 
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    ).unwrap();

    loop {
        let event = StormProcess::wait_for_event().unwrap();
        println!("tornado: got event {:?}", event);
        // process.process_event(&wrapper.event);
        console_client.process_event(&process, &event, &mut state);
        tornado_server.process_event(&mut process, &event, &mut state);
    }

    process.end();
}
