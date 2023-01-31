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

use core::sync::atomic::{AtomicBool, Ordering};

pub struct FromChannel<T> {
    value: T,
}

impl<T> FromChannel<T> {
    pub fn new (value: T) -> Self {
        Self {
            value: value,
        }
    }
}
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

impl ChannelMessageHeader {
    pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);

    pub fn get_payload_address(message: *mut ChannelMessageHeader) -> *mut u8 {
        unsafe { message.offset(mem::size_of::<ChannelMessageHeader>() as isize) as *mut u8 }
    }
}

struct ChannelLock {
    channel_header: *const ChannelHeader,
}

impl ChannelLock {
    pub unsafe fn get(channel: &ConsoleChannel) -> Self {
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
        if !is_server {
            let channel_header = channel_address as *mut ChannelHeader;
            (*channel_header).lock.store(false, Ordering::Relaxed);
            (*channel_header).channel_magic = ChannelHeader::MAGIC;
            (*channel_header).protocol_name[0] = 7;
            (*channel_header).protocol_name[1] = 'c' as u8;
            (*channel_header).protocol_name[2] = 'o' as u8;
            (*channel_header).protocol_name[3] = 'n' as u8;
            (*channel_header).protocol_name[4] = 's' as u8;
            (*channel_header).protocol_name[5] = 'o' as u8;
            (*channel_header).protocol_name[6] = 'l' as u8;
            (*channel_header).protocol_name[7] = 'e' as u8;
            (*channel_header).protocol_version = ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
                is_preview: false,
                preview_version: 0,
            };
            (*channel_header).first_message_offset = 0;
            (*channel_header).last_message_offset = 0;
            (*channel_header).number_of_messages = 0;
            (*channel_header).is_writing = false;
        }
        ConsoleChannel {
            channel_address: channel_address,
        }
    }

    pub unsafe fn check_compatible(&self) -> bool {
        let channel_header = self.channel_address as *mut ChannelHeader;
        (*channel_header).channel_magic == ChannelHeader::MAGIC && (*channel_header).protocol_version.major == 1 && (*channel_header).protocol_name[0] == 7 && (*channel_header).protocol_name[1] == 'c' as u8 && (*channel_header).protocol_name[2] == 'o' as u8 && (*channel_header).protocol_name[3] == 'n' as u8 && (*channel_header).protocol_name[4] == 's' as u8 && (*channel_header).protocol_name[5] == 'o' as u8 && (*channel_header).protocol_name[6] == 'l' as u8 && (*channel_header).protocol_name[7] == 'e' as u8
    }

    pub unsafe fn prepare_message(&self, message_id: u64, replace_pending: bool) -> *mut ChannelMessageHeader {
        let channel_header = self.channel_address as *mut ChannelHeader;
        let lock = ChannelLock::get(self);
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        assert!(!(*channel_header).is_writing);
        let mut message: *mut ChannelMessageHeader;
        if (*channel_header).number_of_messages == 0 {
            (*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();
            (*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();
            message = self.channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;
            (*message).previous_message_offset = 0;
        }
        else {
            let last_message_offset = (*channel_header).last_message_offset;
            let last_message = self.channel_address.offset(last_message_offset as isize) as *mut ChannelMessageHeader;
            (*last_message).next_message_offset = (*channel_header).last_message_offset + (*last_message).message_length;
            message = self.channel_address.offset((*last_message).next_message_offset as isize) as *mut ChannelMessageHeader;
            (*message).previous_message_offset = last_message_offset;
        }
        (*channel_header).is_writing = true;
        (*message).message_magic = ChannelMessageHeader::MAGIC;
        (*message).message_id = message_id;
        (*message).replace_pending = replace_pending;
        (*message).message_length = 0;
        (*message).next_message_offset = 0;
        message
    }

    pub unsafe fn commit_message(&self, message_payload_size: usize) {
        let channel_header = self.channel_address as *mut ChannelHeader;
        let lock = ChannelLock::get(self);
        let last_message = self.channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        assert!((*channel_header).is_writing);
        #[cfg(debug)]
        assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);
        (*channel_header).is_writing = false;
        (*channel_header).number_of_messages = (*channel_header).number_of_messages + 1;
        (*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;
    }

    pub unsafe fn find_specific_message(&self, message_id: u64) -> Option<*mut ChannelMessageHeader> {
        let channel_header = self.channel_address as *mut ChannelHeader;
        let lock = ChannelLock::get(self);
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        if (*channel_header).number_of_messages == 0 {
            None
        }
        else {
            let first_message = self.channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
            #[cfg(debug)]
            assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
            let iter = first_message;
            while (*iter).message_id != message_id && (*iter).next_message_offset != 0 {
                let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
            }
            if (*iter).message_id == message_id {
                Some(iter)
            }
            else {
                None
            }
        }
    }

    pub unsafe fn find_message(&self) -> Option<*mut ChannelMessageHeader> {
        let channel_header = self.channel_address as *mut ChannelHeader;
        let lock = ChannelLock::get(self);
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        if (*channel_header).number_of_messages == 0 {
            None
        }
        else {
            let first_message = self.channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
            #[cfg(debug)]
            assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
            if !(*first_message).replace_pending {
                Some(first_message)
            }
            else {
                let mut last_of_kind = first_message;
                let iter = first_message;
                while (*iter).next_message_offset != 0 {
                    let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                    if (*iter).message_id == (*first_message).message_id {
                        last_of_kind = iter;
                    }
                }
                let iter = first_message;
                while (*iter).next_message_offset != 0 {
                    if (*iter).message_id == (*first_message).message_id && iter != last_of_kind {
                        assert!((*channel_header).number_of_messages > 1);
                        self.unlink_message(iter, true);
                    }
                    let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                }
                Some(last_of_kind)
            }
        }
    }

    pub unsafe fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool) {
        let channel_header = self.channel_address as *mut ChannelHeader;
        let lock = if lock_held { None } else { Some(ChannelLock::get(self)) };
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        #[cfg(debug)]
        assert!((*message).message_magic == ChannelMessageHeader::MAGIC);
        if (*message).previous_message_offset == 0 {
            (*channel_header).first_message_offset = (*message).next_message_offset;
        }
        else {
            let previous_message = self.channel_address.offset((*message).previous_message_offset as isize) as *mut ChannelMessageHeader;
            (*previous_message).next_message_offset = (*message).next_message_offset;
        }
        if (*message).next_message_offset == 0 {
            (*channel_header).last_message_offset = (*message).previous_message_offset;
        }
        else {
            let next_message = self.channel_address.offset((*message).next_message_offset as isize) as *mut ChannelMessageHeader;
            (*next_message).previous_message_offset = (*message).previous_message_offset;
        }
        (*channel_header).number_of_messages = (*channel_header).number_of_messages - 1;
    }
}


