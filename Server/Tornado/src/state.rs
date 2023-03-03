use alloc::collections::BTreeMap;
use library_chaos::{ChannelHandle, ServiceHandle, StormProcess};
use protocol_console::{ConsoleClient, ConsoleClientEvent, ConsoleClientObserver};
use protocol_tornado::{TornadoServer, TornadoServerObserver, TornadoServerRequest};

struct ClientState {}

impl ClientState {
    pub fn new() -> Self {
        ClientState {}
    }
}

pub struct ServerState {
    process: StormProcess,
    tornado_server: TornadoServer,
    console_client: ConsoleClient,
    clients: BTreeMap<ChannelHandle, ClientState>,
}

impl ServerState {
    pub fn new(process: StormProcess, tornado_server: TornadoServer, console_client: ConsoleClient) -> Self {
        Self {
            process: process,
            tornado_server: tornado_server,
            console_client: console_client,
            clients: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) {
        // main event loop
        loop {
            let event = StormProcess::wait_for_event().unwrap();
            self.console_client.process_event(&self.process, &event, self);
            self.tornado_server.process_event(&mut self.process, &event, self);
        }
    }

    pub fn add_client(&mut self, handle: ChannelHandle) {
        self.clients.insert(handle, ClientState::new());
    }

    pub fn remove_client(&mut self, handle: ChannelHandle) {
        self.clients.remove(&handle);
    }
}

impl ConsoleClientObserver for ServerState {
    fn handle_console_event(&mut self, channel_handle: ChannelHandle, event: ConsoleClientEvent) {
        match event {
            ConsoleClientEvent::PointerMoved(parameters) => {
                println!(
                    "moved to {} {}",
                    parameters.position.x, parameters.position.y
                );
            }
            _ => {}
        }
    }
}

impl TornadoServerObserver for ServerState {
    fn handle_tornado_client_connected(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
    ) {
        println!("handle_tornado_client_connected");
        self.add_client(channel_handle);
    }

    fn handle_tornado_client_disconnected(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
    ) {
        println!("handle_tornado_client_disconnected");
        self.remove_client(channel_handle);
    }

    fn handle_tornado_request(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
        request: TornadoServerRequest,
    ) {
        println!("handle_tornado_request");

        match request {
            TornadoServerRequest::SetRenderTree(parameters) => {
                println!("setting render tree");
            }
            _ => {}
        }
    }
}
