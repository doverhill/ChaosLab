extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_data;
extern crate sdl2;

mod application;
use application::ServerApplication;
mod helpers;
mod dirty_patches;

use library_chaos::*;
use protocol_console::*;
use protocol_data::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::new("HostServer.Console").unwrap();

    // set up console service
    let console_server = ConsoleServer::create(&mut process, "Chaos", "Console host server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    // set up data service
    let data_server = DataServer::create(&mut process, "Chaos", "Data host server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    let mut server_application = ServerApplication::new(process, console_server, data_server);
    server_application.run();

    // process.borrow().end();
}
