#[derive(PartialEq, Eq, Debug)]
pub enum Action {
    Unknown = 0,
    Connect = 1,
    Open = 2,
    Close = 3,
    Read = 4,
    Write = 5
}

impl Action {
    pub fn from_i32(value: i32) -> Option<Action> {
        match value {
            0 => Some(Action::Unknown),
            1 => Some(Action::Connect),
            2 => Some(Action::Open),
            3 => Some(Action::Close),
            4 => Some(Action::Read),
            5 => Some(Action::Write),
            _ => None
        }
    }
}
