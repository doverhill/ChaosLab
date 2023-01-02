#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

struct ProtocolVersion {
    major: u16,
    minor: u16,
    patch: u16,
    is_preview: bool,
    preview_version: u16,
}

struct ChannelHeader {
    lock: AtomicBool,
    channel_magic: u64,
    protocol_name: [u8; 32],
    protocol_version: ProtocolVersion,
    first_message_offset: usize,
    last_message_offset: usize,
    number_of_messages: usize,
    is_writing: bool,
}

impl ChannelHeader {
    pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'C' as u8, 'H' as u8, 'A' as u8, 'N' as u8, 'N' as u8, 'E' as u8, 'L' as u8]);
}

pub struct ChannelMessageHeader {
    message_magic: u64,
    message_id: u64,
    message_length: usize,
    previous_message_offset: usize,
    next_message_offset: usize,
    replace_pending: bool,
}

impl ChannelHeader {
    pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);
}

struct ChannelLock {
    channel_header: *const ChannelHeader,
}

impl ChannelLock {
    pub fn get(channel: &ConsoleChannel) -> Self {
        let channel_header = channel.channel_address as *const ChannelHeader;
        while (*channel_header).lock.swap(true, Ordering::Acquire) {}
        Self {
            channel_header: channel_header
        }
    }
}

impl Drop for ChannelLock {
    fn drop(&mut self) {
        unsafe {
            (*self.channel_header).lock.swap(false, Ordering::Release);
        }
    }
}

pub struct ConsoleChannel {
    channel_address: *mut u8,
}

impl ConsoleChannel {
    pub unsafe fn new(channel_address: *mut u8, is_server: bool) -> Self {
    }
}


