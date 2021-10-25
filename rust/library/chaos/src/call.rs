use crate::{ syscalls, handle::Handle, error::Error };

#[allow(dead_code)]
pub struct Call {
    pub channel_handle: Handle,
    pub function: u64
}

impl Call {
    pub fn then(self, _callback: fn(*mut u8) -> ()) -> Call {
        self
    }

    pub fn orelse(self, _callback: fn(Error) -> ()) -> Call {
        self
    }

    pub fn call(self) -> Option<Error> {
        syscalls::channel_signal(self.channel_handle, self.function)
    }
}