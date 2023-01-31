use crate::{ServiceHandle, ChannelHandle};

pub struct Service<'a> {
    pub on_connected: Option<Box<dyn Fn(ServiceHandle, ChannelHandle) + 'a>>,
}

impl<'a> Service<'a> {
    pub fn new() -> Self {
        Service {
            on_connected: None,
        }
    }
}
