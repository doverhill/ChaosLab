use library_chaos::{ServiceHandle, ChannelHandle, ServiceObserver, ChannelObserver, StormProcess};
use protocol_console::{ConsoleClientEvent, ConsoleClientObserver};
use protocol_tornado::{TornadoServerRequest, TornadoServerObserver};
use alloc::collections::BTreeMap;

#[derive(PartialEq)]
struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

#[derive(PartialEq)]
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

impl<'a> ServiceObserver for ServerState {
    fn handle_service_connected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle,) {
        StormProcess::<Self, Self>::emit_debug("handle_service_connected");
    }
}

impl<'a> ChannelObserver for ServerState {
    fn handle_channel_messaged(&self, channel_handle: ChannelHandle, message_id: u64) {
        StormProcess::<Self, Self>::emit_debug("handle_channel_messaged");
    }

    fn handle_channel_destroyed(&self, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_channel_destroyed");
    }
}

impl<'a> ConsoleClientObserver for ServerState {
    fn handle_console_event(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle, event: ConsoleClientEvent) {
        StormProcess::<Self, Self>::emit_debug("handle_console_event");
    }
}

impl<'a> TornadoServerObserver for ServerState {
    fn handle_tornado_client_connected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_tornado_client_connected");
    }

    fn handle_tornado_client_disconnected(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_tornado_client_disconnected");
    }

    fn handle_tornado_request(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: TornadoServerRequest) {
        StormProcess::<Self, Self>::emit_debug("handle_tornado_request");
    }
}
