#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    None = 0,
    NotFound = 1,
    PermissionDenied = 2,
    NotImplemented = 3,
    Timeout = 4,
    Cancelled = 5,
    AlreadyExists = 6,
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
            6 => Error::AlreadyExists,
            7 => Error::General,
            _ => Error::General
        }
    }

    pub fn to_i32(error: &Error) -> i32 {
        match error {
            Error::None => 0,
            Error::NotFound => 1,
            Error::PermissionDenied => 2,
            Error::NotImplemented => 3,
            Error::Timeout => 4,
            Error::Cancelled => 5,
            Error::AlreadyExists => 6,
            Error::General => 7
        }
    }
}