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
    pub message_id: u64,
    pub call_id: u64,
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
    pub unsafe fn get(channel_address: *mut u8) -> Self {
        let channel_header = channel_address as *const ChannelHeader;
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
    rx_channel_address: *mut u8,
    tx_channel_address: *mut u8,
    call_id: u64,
}

impl ConsoleChannel {
    pub fn new(channel_address_0: *mut u8, channel_address_1: *mut u8, is_server: bool) -> Self {
        unsafe {
            if is_server {
                ConsoleChannel {
                    rx_channel_address: channel_address_0,
                    tx_channel_address: channel_address_1,
                    call_id: 1,
                }
            }
            else {
                Self::initialize(channel_address_0);
                Self::initialize(channel_address_1);
                ConsoleChannel {
                    rx_channel_address: channel_address_1,
                    tx_channel_address: channel_address_0,
                    call_id: 1,
                }
            }
        }
    }

    unsafe fn initialize(channel_address: *mut u8) {
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

    pub fn check_compatible(&self) -> bool {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let rx_compatible = (*channel_header).channel_magic == ChannelHeader::MAGIC && (*channel_header).protocol_version.major == 1 && (*channel_header).protocol_name[0] == 7 && (*channel_header).protocol_name[1] == 'c' as u8 && (*channel_header).protocol_name[2] == 'o' as u8 && (*channel_header).protocol_name[3] == 'n' as u8 && (*channel_header).protocol_name[4] == 's' as u8 && (*channel_header).protocol_name[5] == 'o' as u8 && (*channel_header).protocol_name[6] == 'l' as u8 && (*channel_header).protocol_name[7] == 'e' as u8;
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let tx_compatible = (*channel_header).channel_magic == ChannelHeader::MAGIC && (*channel_header).protocol_version.major == 1 && (*channel_header).protocol_name[0] == 7 && (*channel_header).protocol_name[1] == 'c' as u8 && (*channel_header).protocol_name[2] == 'o' as u8 && (*channel_header).protocol_name[3] == 'n' as u8 && (*channel_header).protocol_name[4] == 's' as u8 && (*channel_header).protocol_name[5] == 'o' as u8 && (*channel_header).protocol_name[6] == 'l' as u8 && (*channel_header).protocol_name[7] == 'e' as u8;
            rx_compatible && tx_compatible
        }
    }

    pub fn prepare_message(&mut self, message_id: u64, replace_pending: bool) -> (u64, *mut ChannelMessageHeader) {
        let call_id = self.call_id;
        self.call_id += 1;
        unsafe {
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get(self.tx_channel_address);
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            assert!(!(*channel_header).is_writing);
            let mut message: *mut ChannelMessageHeader;
            if (*channel_header).number_of_messages == 0 {
                (*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();
                (*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();
                message = self.tx_channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;
                (*message).previous_message_offset = 0;
            }
            else {
                let last_message_offset = (*channel_header).last_message_offset;
                let last_message = self.tx_channel_address.offset(last_message_offset as isize) as *mut ChannelMessageHeader;
                (*last_message).next_message_offset = (*channel_header).last_message_offset + (*last_message).message_length;
                message = self.tx_channel_address.offset((*last_message).next_message_offset as isize) as *mut ChannelMessageHeader;
                (*message).previous_message_offset = last_message_offset;
            }
            (*channel_header).is_writing = true;
            (*message).message_magic = ChannelMessageHeader::MAGIC;
            (*message).message_id = message_id;
            (*message).call_id = call_id;
            (*message).replace_pending = replace_pending;
            (*message).message_length = 0;
            (*message).next_message_offset = 0;
            (call_id, message)
        }
    }

    pub fn commit_message(&self, message_payload_size: usize) {
        unsafe {
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get(self.tx_channel_address);
            let last_message = self.tx_channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            assert!((*channel_header).is_writing);
            #[cfg(debug)]
            assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);
            (*channel_header).is_writing = false;
            (*channel_header).number_of_messages = (*channel_header).number_of_messages + 1;
            (*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;
        }
    }

    pub fn find_specific_message(&self, call_id: u64) -> Option<*mut ChannelMessageHeader> {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get(self.rx_channel_address);
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            if (*channel_header).number_of_messages == 0 {
                None
            }
            else {
                let first_message = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
                #[cfg(debug)]
                assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
                let iter = first_message;
                while (*iter).call_id != call_id && (*iter).next_message_offset != 0 {
                    let iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                }
                if (*iter).call_id == call_id {
                    Some(iter)
                }
                else {
                    None
                }
            }
        }
    }

    pub fn find_message(&self) -> Option<*mut ChannelMessageHeader> {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get(self.rx_channel_address);
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            if (*channel_header).number_of_messages == 0 {
                None
            }
            else {
                let first_message = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
                #[cfg(debug)]
                assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
                if !(*first_message).replace_pending {
                    Some(first_message)
                }
                else {
                    let mut last_of_kind = first_message;
                    let iter = first_message;
                    while (*iter).next_message_offset != 0 {
                        let iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
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
                        let iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                    }
                    Some(last_of_kind)
                }
            }
        }
    }

    pub fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool) {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let lock = if lock_held { None } else { Some(ChannelLock::get(self.rx_channel_address)) };
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            #[cfg(debug)]
            assert!((*message).message_magic == ChannelMessageHeader::MAGIC);
            if (*message).previous_message_offset == 0 {
                (*channel_header).first_message_offset = (*message).next_message_offset;
            }
            else {
                let previous_message = self.rx_channel_address.offset((*message).previous_message_offset as isize) as *mut ChannelMessageHeader;
                (*previous_message).next_message_offset = (*message).next_message_offset;
            }
            if (*message).next_message_offset == 0 {
                (*channel_header).last_message_offset = (*message).previous_message_offset;
            }
            else {
                let next_message = self.rx_channel_address.offset((*message).next_message_offset as isize) as *mut ChannelMessageHeader;
                (*next_message).previous_message_offset = (*message).previous_message_offset;
            }
            (*channel_header).number_of_messages = (*channel_header).number_of_messages - 1;
        }
    }
}


