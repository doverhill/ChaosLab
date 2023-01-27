extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

use library_chaos::{StormEvent, StormHandle, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

struct ClientState {

}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

struct GlobalState {
    clients: HashMap<StormHandle, ClientState>
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState { clients: HashMap::new() }
    }

    pub fn add_client(&self, channel_handle: StormHandle) {

    }

    pub fn remove_client(&self, channel_handle: StormHandle) {
        
    }
}

fn main() {
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let mut state = State {};

    // connect to console
    let console_client = ConsoleClient::connect_first(process, CONSOLE_PROTOCOL_NAME).unwrap();

    console_client.write_text("Hello from Tornado!");

    // set up service
    let tornado_server = TornadoServer::create(process, "Chaos", "Tornado server", Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()).unwrap();

    // set up logic
    console_client.on_pointer_moved(|| {
        println!("pointer moved");
    });



    tornado_server.on_connect(|channel_handle| {
        state.add_client(channel_handle);
    });

    tornado_server.on_disconnect(|channel_handle| {
        state.remove_client(channel_handle);
    });

    // run
    process.run();
}

