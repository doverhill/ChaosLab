use library_chaos::{ServiceHandle, ChannelHandle, ServiceObserver, ChannelObserver, StormProcess};
use protocol_console::{ConsoleServerRequest, ConsoleServerObserver};
use std::collections::HashMap;

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
    clients: HashMap<ChannelHandle, ClientState>,
}

impl ServerState {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    pub fn add_client(&mut self, handle: ChannelHandle) {
        self.clients.insert(handle, ClientState::new());
    }

    pub fn remove_client(&mut self, handle: ChannelHandle) {
        self.clients.remove(&handle);
    }

    pub fn get_first_client_handle(&self) -> Option<&ChannelHandle> {
        self.clients.keys().next()
    }
}

impl<'a> ServiceObserver for ServerState {
    fn handle_service_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle,) {
        StormProcess::<Self, Self>::emit_debug("handle_service_connected");
    }
}

impl<'a> ChannelObserver for ServerState {
    fn handle_channel_messaged(&mut self, channel_handle: ChannelHandle, message_id: u64) {
        StormProcess::<Self, Self>::emit_debug("handle_channel_messaged");
    }

    fn handle_channel_destroyed(&mut self, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_channel_destroyed");
    }
}

impl<'a> ConsoleServerObserver for ServerState {
    fn handle_console_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_console_client_connected");
        self.add_client(channel_handle);
    }

    fn handle_console_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        StormProcess::<Self, Self>::emit_debug("handle_console_client_disconnected");
        self.remove_client(channel_handle);
    }

    fn handle_console_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: ConsoleServerRequest) {
        StormProcess::<Self, Self>::emit_debug("handle_console_request");
    }
}
