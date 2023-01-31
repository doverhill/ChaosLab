use library_chaos::{StormEvent, ChannelHandle, StormProcess};
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
