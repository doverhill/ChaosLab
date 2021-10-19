#[derive(PartialEq, Eq)]
pub enum Error {
    None = 0,
    NotFound = 1,
    PermissionDenied = 2,
    NotImplemented = 3,
    Timeout = 4,
    Cancelled = 5,
    Duplicate = 6
}

impl Error {
    pub fn from_i32(value: i32) -> Option<Error> {
        match value {
            0 => Some(Error::None),
            1 => Some(Error::NotFound),
            2 => Some(Error::PermissionDenied),
            3 => Some(Error::NotImplemented),
            4 => Some(Error::Timeout),
            5 => Some(Error::Cancelled),
            6 => Some(Error::Duplicate),
            _ => None
        }
    }
}