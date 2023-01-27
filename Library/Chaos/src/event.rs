use crate::{ StormHandle };

#[derive(PartialEq, Eq, Debug)]
pub enum StormAction {
    None = 0,
    ServiceConnected = 1,
    ChannelMessaged = 2,
    ChannelDestroyed = 3
}

impl StormAction {
    pub fn to_i32(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::ServiceConnected => 1,
            Self::ChannelMessaged => 2,
            Self::ChannelDestroyed => 3
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::ServiceConnected,
            2 => Self::ChannelMessaged,
            3 => Self::ChannelDestroyed,
            _ => Self::None
        }
    }
}

#[derive(Debug)]
pub struct StormEvent {
    handle: StormHandle,
    action: StormAction,
    argument_handle: StormHandle,
    parameter: u64
}