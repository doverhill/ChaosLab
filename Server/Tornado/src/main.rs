// #![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::ServerState;

use alloc::sync::Arc;
use core::cell::RefCell;
use library_chaos::{ChannelHandle, StormEvent, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

struct ServerApplication {
    process: StormProcess,
    state: ServerState,
}

impl ServerApplication {
    pub fn new() -> Self {
        // let mut process = StormProcess::new("Server.Tornado").unwrap();
        // let mut state = Arc::new(RefCell::new(ServerState::new()));
        // let mut state = ServerState::new();

        Self {
            process: StormProcess::new("Server.Tornado").unwrap(),
            state: ServerState::new(),
        }
    }

    pub fn run(&mut self) {
        // connect to console
        // let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

        // console_client.write_text(&WriteTextParameters {
        //     text: "Hello from Tornado!".to_string(),
        // });

        // console_client.attach_observer(self);
        // console_client.on_pointer_moved(|_| {
        //     StormProcess::emit_debug("tornado: pointer moved");
        // });

        // set up service
        let mut tornado_server = TornadoServer::create(
            &mut process,
            "Chaos",
            "Tornado server",
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        )
        .unwrap();

        tornado_server.attach_observer(self);

        // tornado_server.on_client_connected(|channel_handle| {
        //     StormProcess::emit_debug("tornado: client connected");
        //     state.borrow_mut().add_client(channel_handle);
        // });

        // tornado_server.on_client_disconnected(|channel_handle| {
        //     StormProcess::emit_debug("tornado: client disconnected");
        //     state.borrow_mut().remove_client(channel_handle);
        // });

        // run
        process.run();
    }
}

fn main() {
    let app = ServerApplication::new();
    app.run();
}
