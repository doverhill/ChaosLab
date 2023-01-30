#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ServiceHandle(pub u64);

impl ServiceHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ChannelHandle(pub u64);

impl ChannelHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}
