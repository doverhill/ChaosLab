// #![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::ServerState;

use library_chaos::{ChannelHandle, ChannelObserver, ServiceHandle, ServiceObserver, StormProcess};
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;

#[derive(PartialEq)]
struct ServerApplication<'a> {
    process: StormProcess<'a, Self, Self>,
    state: ServerState,
    console: ConsoleClient<'a, Self, Self, Self>,
    server: TornadoServer<'a, Self, Self, Self>,
}

impl<'a> ServiceObserver for ServerApplication<'a> {
    fn handle_service_connected(
        &self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
    ) {
    }
}

impl<'a> ChannelObserver for ServerApplication<'a> {
    fn handle_channel_destroyed(&self, channel_handle: ChannelHandle) {}
    fn handle_channel_messaged(&self, channel_handle: ChannelHandle, message_id: u64) {
        
    }
}

impl<'a> ConsoleClientObserver for ServerApplication<'a> {
    fn handle_console_event(service_handle: ServiceHandle, channel_handle: ChannelHandle, event: ConsoleClientEvent) {
        
    }
}

impl<'a> TornadoServerObserver for ServerApplication<'a> {
    fn handle_tornado_client_connected(service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        
    }

    fn handle_tornado_client_disconnected(service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        
    }

    fn handle_tornado_request(service_handle: ServiceHandle, channel_handle: ChannelHandle, request: TornadoServerRequest) {
        
    }
}

impl<'a> ServerApplication<'a> {
    pub fn new() -> Self {
        // let mut process = StormProcess::new("Server.Tornado").unwrap();
        // let mut state = Arc::new(RefCell::new(ServerState::new()));
        // let mut state = ServerState::new();

        let mut process = StormProcess::new("Server.Tornado").unwrap();
        let state = ServerState::new();
        let mut console = ConsoleClient::connect_first(&mut process).unwrap();
        let mut server = TornadoServer::create(
            &mut process,
            "Chaos",
            "Tornado server",
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        )
        .unwrap();

        let app = Self {
            process: process,
            state: state,
            console: console,
            server: server,
        };

        app.console.attach_observer(&app);
        server.attach_observer(&app);

        app
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
        // let mut ;

        // self.server.attach_observer(self);

        // tornado_server.on_client_connected(|channel_handle| {
        //     StormProcess::emit_debug("tornado: client connected");
        //     state.borrow_mut().add_client(channel_handle);
        // });

        // tornado_server.on_client_disconnected(|channel_handle| {
        //     StormProcess::emit_debug("tornado: client disconnected");
        //     state.borrow_mut().remove_client(channel_handle);
        // });

        // run
        self.process.run();
    }
}

fn main() {
    let mut process = StormProcess::new("Server.Tornado").unwrap();
    let state = ServerState::new();
    let mut console = ConsoleClient::connect_first(&mut process).unwrap();
    let mut server = TornadoServer::create(
        &mut process,
        "Chaos",
        "Tornado server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    )
    .unwrap();

    let app = Self {
        process: process,
        state: state,
        console: console,
        server: server,
    };

    app.console.attach_observer(&app);
    server.attach_observer(&app);

    let app = ServerApplication::new();
    app.run();
}
