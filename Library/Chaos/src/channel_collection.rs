use crate::{ StormHandle };

pub struct ChannelCollection {
    handles: Vec<StormHandle>
}

impl ChannelCollection {
    pub fn new() -> Self {
        ChannelCollection { handles: vec![] }
    }
}