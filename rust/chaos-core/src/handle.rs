use std::fmt;

pub struct Handle {
    id: u64,
    on_connect: Option<fn() -> ()>
}

impl Handle {
    pub fn new(id: u64) -> Handle {
        Handle {
            id: id,
            on_connect: None
        }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "h{}", self.id)
    }
}