use library_chaos::{StormEvent, StormHandle, StormProcess};
use alloc::collections::BTreeMap;

struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

struct GlobalState {
    clients: BTreeMap<StormHandle, ClientState>
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState { clients: BTreeMap::new() }
    }

    pub fn add_client(&self, channel_handle: StormHandle) {
        self.clients.insert(channel_handle, ClientState::new());
    }

    pub fn remove_client(&self, channel_handle: StormHandle) {
        self.clients.remove(&channel_handle);
    }
}
