extern crate library_chaos;
extern crate protocol_console;
extern crate sdl2;

mod application;
use application::ServerApplication;
mod helpers;

use library_chaos::{StormEvent, StormProcess, StormError};
use protocol_console::*;
use uuid::Uuid;
use core::cell::RefCell;
use std::rc::Rc;

fn main() {
    let mut process = StormProcess::new("HostServer.Console").unwrap();

    // set up service
    let mut console_server = ConsoleServer::create(
        &mut process,
        "Chaos",
        "SDL console host server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    ).unwrap();

    let mut server_application = ServerApplication::new(process, console_server);
    server_application.run();

    // process.borrow().end();
}
