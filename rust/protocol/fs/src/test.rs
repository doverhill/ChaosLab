use crate::{
    types::{ DirectoryChild },
    ipc::Functions
};
use chaos::{ channel::Channel, channel_iterator::ChannelIterator };

struct DirectoryListParameters {
    full_path: [u8; 100]
}

pub fn directory_list(channel: &Channel, full_path: &str) -> ChannelIterator<DirectoryChild> {
    unsafe {
        let parameters: (*mut DirectoryListParameters)channel.channel_pointer;

        // copy arguments

        // signal channel
        process::channel_interface(channel.channel_handle, Functions.DirectoryList).
            
    }
}
