use std::fmt;

pub struct Handle {
    pub id: u64
}

impl Handle {
    pub fn new(id: u64) -> Handle {
        Handle {
            id: id
        }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "h{}", self.id)
    }
}