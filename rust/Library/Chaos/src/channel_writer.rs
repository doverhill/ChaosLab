use crate::ChannelHeader;

pub struct ChannelWriter {
    channel_buffer_pointer: *mut ChannelHeader,
}

impl ChannelWriter {
    pub fn new(channel_buffer_pointer: *mut u8) -> Self {
        unsafe {
            ChannelWriter {
                channel_buffer_pointer: core::mem::transmute(channel_buffer_pointer),
            }
        }
    }

    pub fn

    pub fn write(&mut self) {}
}
