use crate::{ServiceHandle, ChannelHandle};

pub struct Service {
    pub on_connected: Option<Box<dyn Fn(ServiceHandle, ChannelHandle)>>,
}

impl Service {
    pub fn new() -> Self {
        Service {
            on_connected: None,
        }
    }
}
