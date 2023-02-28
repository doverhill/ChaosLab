use library_chaos::{ServiceHandle, ChannelHandle, StormProcess};
use protocol_console::{ConsoleServerRequest, ConsoleServerObserver};
use std::collections::HashMap;

struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

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

impl ConsoleServerObserver for ServerState {
    fn handle_console_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_console_client_connected");
        self.add_client(channel_handle);
    }

    fn handle_console_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_console_client_disconnected");
        self.remove_client(channel_handle);
    }

    fn handle_console_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: ConsoleServerRequest) {
        println!("handle_console_request");

        match request {
            ConsoleServerRequest::WriteText(parameters) => {
                println!("write_text");
                println!("write {} to console", parameters.text);
            },
            _ => {}
        }
    }
}
