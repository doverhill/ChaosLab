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

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();
    console_client.write_text(&WriteTextParameters { text: "hello console".to_string() });

    {
        let console_info = console_client.get_capabilities(&process).unwrap();
        println!("tornado: {}x{}", console_info.framebuffer_size.width, console_info.framebuffer_size.height);
    }

    // create tornado service
    let tornado_server = TornadoServer::create(
        &mut process, 
        "Chaos", 
        "Tornado server", 
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    ).unwrap();

    let mut application = ServerState::new(process, tornado_server, console_client);
    application.run();

    // process.end();
}
