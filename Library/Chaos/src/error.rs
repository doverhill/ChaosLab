#[derive(PartialEq, Eq, Debug)]
pub enum StormError {
    None = 0,
    NotFound = 1,
    PermissionDenied = 2,
    NotImplemented = 3,
    Timeout = 4,
    Cancelled = 5,
    AlreadyExists = 6,
    General = 7
}

impl StormError {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::NotFound,
            2 => Self::PermissionDenied,
            3 => Self::NotImplemented,
            4 => Self::Timeout,
            5 => Self::Cancelled,
            6 => Self::AlreadyExists,
            7 => Self::General,
            _ => Self::General
        }
    }

    pub fn to_i32(error: Self) -> i32 {
        match error {
            Self::None => 0,
            Self::NotFound => 1,
            Self::PermissionDenied => 2,
            Self::NotImplemented => 3,
            Self::Timeout => 4,
            Self::Cancelled => 5,
            Self::AlreadyExists => 6,
            Self::General => 7
        }
    }
}