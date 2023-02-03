#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ServiceHandle(pub u64);

impl ServiceHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ChannelHandle(pub u64);

impl ChannelHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

// pub trait ServiceObserver {
//     fn handle_service_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);
//     // fn handle_service_disconnected(service_handle: ServiceHandle, channel_handle: ChannelHandle);
// }

// pub trait ChannelObserver {
//     fn handle_channel_messaged(&mut self, channel_handle: ChannelHandle, message_id: u64);
//     fn handle_channel_destroyed(&mut self, channel_handle: ChannelHandle);
// }
