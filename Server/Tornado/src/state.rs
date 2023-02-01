use library_chaos::{StormEvent, ChannelHandle, StormProcess};
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
