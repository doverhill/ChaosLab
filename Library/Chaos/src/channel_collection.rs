use crate::{ Handle };

pub struct ChannelCollection {
    handles: Vec<Handle>
}

impl ChannelCollection {
    pub fn new() -> Self {
        ChannelCollection { handles: vec![] }
    }
}