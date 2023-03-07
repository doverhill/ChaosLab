extern crate library_chaos;
extern crate protocol_console;
extern crate sdl2;

mod application;
use application::ServerApplication;
mod helpers;

use library_chaos::*;
use protocol_console::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::new("HostServer.Console").unwrap();

    // set up service
    let console_server = ConsoleServer::create(
        &mut process,
        "Chaos",
        "Console host server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    ).unwrap();

    let mut server_application = ServerApplication::new(process, console_server);
    server_application.run();

    // process.borrow().end();
}
