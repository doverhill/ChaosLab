use crate::{ChannelHandle, ServiceHandle, ServiceSubscribeHandle, TimerHandle, ProcessHandle};

#[derive(PartialEq, Eq, Debug)]
pub enum StormAction {
    // None = 0,
    ServiceConnected = 100,
    ServiceAvailable = 101,

    ChannelSignalled = 200,
    ChannelDestroyed = 201,

    TimerFired = 300,

    ProcessExited = 400,
}

#[derive(Debug, Copy, Clone)]
pub enum StormEvent {
    ServiceConnected(ServiceHandle, ChannelHandle),
    ServiceAvailable(ServiceSubscribeHandle),
    ChannelSignalled(ChannelHandle),
    ChannelDestroyed(ChannelHandle),
    TimerFired(TimerHandle),
    ProcessExited(ProcessHandle),
}
