#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct ServiceHandle(pub u64);

impl ServiceHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct ChannelHandle(pub u64);

impl ChannelHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct ServiceSubscribeHandle(pub u64);

impl ServiceSubscribeHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct TimerHandle(pub u64);

impl TimerHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct ProcessHandle(pub u64);

impl ProcessHandle {
    pub fn raw_handle(&self) -> u64 {
        self.0
    }
}
