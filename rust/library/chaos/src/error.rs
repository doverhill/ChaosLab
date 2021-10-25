#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    None = 0,
    NotFound = 1,
    PermissionDenied = 2,
    NotImplemented = 3,
    Timeout = 4,
    Cancelled = 5,
    Duplicate = 6,
    General = 7
}

impl Error {
    pub fn from_i32(value: i32) -> Error {
        match value {
            0 => Error::None,
            1 => Error::NotFound,
            2 => Error::PermissionDenied,
            3 => Error::NotImplemented,
            4 => Error::Timeout,
            5 => Error::Cancelled,
            6 => Error::Duplicate,
            7 => Error::General,
            _ => Error::General
        }
    }
}