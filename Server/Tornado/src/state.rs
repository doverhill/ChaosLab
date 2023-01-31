use library_chaos::{StormEvent, ChannelHandle, StormProcess};
use alloc::collections::BTreeMap;

struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

pub struct GlobalState {
    clients: BTreeMap<ChannelHandle, ClientState>
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState { clients: BTreeMap::new() }
    }

    pub fn add_client(&mut self, handle: ChannelHandle) {
        self.clients.insert(handle, ClientState::new());
    }

    pub fn remove_client(&mut self, handle: ChannelHandle) {
        self.clients.remove(&handle);
    }
}
