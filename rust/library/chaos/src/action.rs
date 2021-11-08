#[derive(PartialEq, Eq, Debug)]
pub enum Action {
    None = 0,
    ServiceConnected = 1,
    ChannelMessaged = 2
}

impl Action {
    pub fn to_i32(&self) -> i32 {
        match self {
            Action::None => 0,
            Action::ServiceConnected => 1,
            Action::ChannelMessaged => 2
        }
    }

    pub fn from_i32(value: i32) -> Action {
        match value {
            0 => Action::None,
            1 => Action::ServiceConnected,
            2 => Action::ChannelMessaged,
            _ => Action::None
        }
    }
}
