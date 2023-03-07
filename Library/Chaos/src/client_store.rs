use crate::{ChannelHandle, ServiceHandle};
use std::collections::BTreeMap;

pub struct ClientStore<TClient> {
    services: BTreeMap<ServiceHandle, BTreeMap<ChannelHandle, TClient>>,
    client_count: usize,
}

impl<TClient> ClientStore<TClient> {
    pub fn new() -> Self {
        Self {
            services: BTreeMap::new(),
            client_count: 0,
        }
    }

    pub fn add_client(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, client: TClient) {
        if let Some(channels) = self.services.get_mut(&service_handle) {
            channels.insert(channel_handle, client);
        } else {
            let mut channels: BTreeMap<ChannelHandle, TClient> = BTreeMap::new();
            channels.insert(channel_handle, client);
            self.services.insert(service_handle, channels);
        }
        self.client_count += 1;
    }

    pub fn remove_client(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        if let Some(channels) = self.services.get_mut(&service_handle) {
            channels.remove(&channel_handle);
            self.client_count -= 1;
        }
    }

    pub fn get_client(&self, service_handle: ServiceHandle, channel_handle: ChannelHandle) -> Option<&TClient> {
        if let Some(channels) = self.services.get(&service_handle) {
            channels.get(&channel_handle)
        } else {
            None
        }
    }

    pub fn first(&self) -> Option<(&ServiceHandle, &ChannelHandle, &TClient)> {
        for (service_handle, channels) in self.services.iter() {
            for (channel_handle, client) in channels.iter() {
                return Some((service_handle, channel_handle, client));
            }
        }
        None
    }

    pub fn first_matching(&self, predicate: impl Fn(&ServiceHandle, &ChannelHandle, &TClient) -> bool) -> Option<(&ServiceHandle, &ChannelHandle, &TClient)> {
        for (service_handle, channels) in self.services.iter() {
            for (channel_handle, client) in channels.iter() {
                if predicate(service_handle, channel_handle, client) {
                    return Some((service_handle, channel_handle, client));
                }
            }
        }
        None
    }

    pub fn for_each_mut(&mut self, mut callback: impl FnMut(&mut TClient)) {
        for (_, channels) in self.services.iter_mut() {
            for (_, client) in channels.iter_mut() {
                callback(client);
            }
        }
    }

    pub fn client_count(&self) -> usize {
        self.client_count
    }
}
