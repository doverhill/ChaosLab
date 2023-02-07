use crate::{ServiceHandle, ChannelHandle};

#[derive(PartialEq, Eq, Debug)]
pub enum StormAction {
    // None = 0,
    ServiceConnected = 1,
    ChannelSignalled = 2,
    ChannelDestroyed = 3
}

impl StormAction {
    pub fn to_i32(&self) -> i32 {
        match self {
            // Self::None => 0,
            Self::ServiceConnected => 1,
            Self::ChannelSignalled => 2,
            Self::ChannelDestroyed => 3
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            // 0 => Self::None,
            1 => Self::ServiceConnected,
            2 => Self::ChannelSignalled,
            3 => Self::ChannelDestroyed,
            _ => panic!("Unknown action")
        }
    }
}

#[derive(Debug)]
pub enum StormEvent {
    ServiceConnected(ServiceHandle, ChannelHandle),
    ChannelSignalled(ChannelHandle),
    ChannelDestroyed(ChannelHandle)
}
