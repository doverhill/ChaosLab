use library_chaos::{ServiceHandle, ChannelHandle, StormProcess};
use protocol_console::{ConsoleClientEvent, ConsoleClientObserver};
use protocol_tornado::{TornadoServerRequest, TornadoServerObserver};
use alloc::collections::BTreeMap;

struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

pub struct ServerState {
    clients: BTreeMap<ChannelHandle, ClientState>,
}

impl ServerState {
    pub fn new() -> Self {
        Self { clients: BTreeMap::new() }
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
        println!("handle_console_event");
    }
}

impl TornadoServerObserver for ServerState {
    fn handle_tornado_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_tornado_client_connected");
    }

    fn handle_tornado_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_tornado_client_disconnected");
    }

    fn handle_tornado_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: TornadoServerRequest) {
        println!("handle_tornado_request");
    }
}
