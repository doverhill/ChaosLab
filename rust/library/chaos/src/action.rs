#[derive(PartialEq, Eq, Debug)]
pub enum Action {
    None = 0,
    Connect = 1,
    Signal = 2
}

impl Action {
    pub fn from_i32(value: i32) -> Action {
        match value {
            0 => Action::None,
            1 => Action::Connect,
            2 => Action::Signal,
            _ => Action::None
        }
    }
}
