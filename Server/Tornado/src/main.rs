#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod application;
use application::ServerApplication;

use library_chaos::StormProcess;
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Server.Tornado").unwrap();

    // connect to console service
    let console_client = ConsoleClient::connect_first(&mut process).unwrap();
    
    // create tornado service
    let tornado_server = TornadoServer::create(&mut process, "Chaos", "Tornado server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    let mut server_application = ServerApplication::new(process, tornado_server, console_client);
    server_application.run();

    // process.end();
}
