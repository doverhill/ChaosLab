#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;

use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::Deref;
use core::marker::PhantomData;

pub struct FromChannel<T> {
    channel_address: *mut ChannelHeader,
    message_address: *mut ChannelMessageHeader,
    phantom: PhantomData<T>,
}

impl<T> FromChannel<T> {
    pub fn new (channel_address: *mut u8, message_address: *mut ChannelMessageHeader) -> Self {
        Self {
            channel_address: channel_address as *mut ChannelHeader,
            message_address: message_address,
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for FromChannel<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { (ChannelMessageHeader::get_payload_address(self.message_address) as *const T).as_ref().unwrap() }
    }
}

impl<T> Drop for FromChannel<T> {
    fn drop(&mut self) {
        unsafe {
            let lock = ChannelLock::get("from_channel_drop", self.channel_address as *mut u8);
            assert!((*self.message_address).pending_unlink);
            #[cfg(debug)]
            assert!((*self.channel_address).channel_magic == ChannelHeader::MAGIC);
            #[cfg(debug)]
            assert!((*self.message_address).message_magic == ChannelMessageHeader::MAGIC);
            if (*self.message_address).previous_message_offset == 0 {
                (*self.channel_address).first_message_offset = (*self.message_address).next_message_offset;
            }
            else {
                let previous_message = self.channel_address.offset((*self.message_address).previous_message_offset as isize) as *mut ChannelMessageHeader;
                (*previous_message).next_message_offset = (*self.message_address).next_message_offset;
            }
            if (*self.message_address).next_message_offset == 0 {
                (*self.channel_address).last_message_offset = (*self.message_address).previous_message_offset;
            }
            else {
                let next_message = self.channel_address.offset((*self.message_address).next_message_offset as isize) as *mut ChannelMessageHeader;
                (*next_message).previous_message_offset = (*self.message_address).previous_message_offset;
            }
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
    is_writing: bool,
    pending_unlink: bool,
}

impl ChannelMessageHeader {
    pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);

    pub fn get_payload_address(message: *mut ChannelMessageHeader) -> *mut u8 {
        unsafe { message.offset(mem::size_of::<ChannelMessageHeader>() as isize) as *mut u8 }
    }
}

struct ChannelLock {
    name: String,
    channel_header: *const ChannelHeader,
}

impl ChannelLock {
    pub unsafe fn get(name: &str, channel_address: *mut u8) -> Self {
        let channel_header = channel_address as *const ChannelHeader;
        while (*channel_header).lock.swap(true, Ordering::Acquire) {}
        Self {
            name: name.to_string(),
            channel_header: channel_header
        }
    }
}

impl Drop for ChannelLock {
    fn drop(&mut self) {
        unsafe {
            (*self.channel_header).lock.store(false, Ordering::Relaxed);
        }
    }
}

pub struct StorageChannel {
    pub rx_channel_address: *mut u8,
    tx_channel_address: *mut u8,
    call_id: u64,
}

impl StorageChannel {
    pub fn new(channel_address_0: *mut u8, channel_address_1: *mut u8, is_server: bool) -> Self {
        unsafe {
            if is_server {
                StorageChannel {
                    rx_channel_address: channel_address_0,
                    tx_channel_address: channel_address_1,
                    call_id: 1,
                }
            }
            else {
                Self::initialize(channel_address_0);
                Self::initialize(channel_address_1);
                StorageChannel {
                    rx_channel_address: channel_address_1,
                    tx_channel_address: channel_address_0,
                    call_id: 1,
                }
            }
        }
    }

    unsafe fn initialize(channel_address: *mut u8) {
        let channel_header = channel_address as *mut ChannelHeader;
        (*channel_header).lock = AtomicBool::new(false);
        (*channel_header).channel_magic = ChannelHeader::MAGIC;
        (*channel_header).protocol_name[0] = 7;
        (*channel_header).protocol_name[1] = 's' as u8;
        (*channel_header).protocol_name[2] = 't' as u8;
        (*channel_header).protocol_name[3] = 'o' as u8;
        (*channel_header).protocol_name[4] = 'r' as u8;
        (*channel_header).protocol_name[5] = 'a' as u8;
        (*channel_header).protocol_name[6] = 'g' as u8;
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
        (*channel_header).is_writing = false;
    }

    pub fn check_compatible(&self) -> bool {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let rx_compatible = (*channel_header).channel_magic == ChannelHeader::MAGIC && (*channel_header).protocol_version.major == 1 && (*channel_header).protocol_name[0] == 7 && (*channel_header).protocol_name[1] == 's' as u8 && (*channel_header).protocol_name[2] == 't' as u8 && (*channel_header).protocol_name[3] == 'o' as u8 && (*channel_header).protocol_name[4] == 'r' as u8 && (*channel_header).protocol_name[5] == 'a' as u8 && (*channel_header).protocol_name[6] == 'g' as u8 && (*channel_header).protocol_name[7] == 'e' as u8;
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let tx_compatible = (*channel_header).channel_magic == ChannelHeader::MAGIC && (*channel_header).protocol_version.major == 1 && (*channel_header).protocol_name[0] == 7 && (*channel_header).protocol_name[1] == 's' as u8 && (*channel_header).protocol_name[2] == 't' as u8 && (*channel_header).protocol_name[3] == 'o' as u8 && (*channel_header).protocol_name[4] == 'r' as u8 && (*channel_header).protocol_name[5] == 'a' as u8 && (*channel_header).protocol_name[6] == 'g' as u8 && (*channel_header).protocol_name[7] == 'e' as u8;
            rx_compatible && tx_compatible
        }
    }

    pub fn dump_rx(&self, text: &str) {
        unsafe {
            let channel_address = self.rx_channel_address;
            println!("DUMPING CHANNEL rx ({}) {:p}", text, channel_address);
            let channel_header = channel_address as *mut ChannelHeader;
            if (*channel_header).first_message_offset == 0 {
                println!("  EMPTY");
            }
            else {
                let mut index = (*channel_header).first_message_offset;
                let mut iter = channel_address.offset((*channel_header).first_message_offset as isize) as *const ChannelMessageHeader;
                'messages: loop {
                    println!("  {}: prev: {}, next: {}, size: {}", index, (*iter).previous_message_offset, (*iter).next_message_offset, (*iter).message_length);
                    if (*iter).next_message_offset == 0 {
                        break 'messages;
                    }
                    index = (*iter).next_message_offset;
                    iter = channel_address.offset((*iter).next_message_offset as isize) as *const ChannelMessageHeader;
                }
            }
        }
    }

    pub fn dump_tx(&self, text: &str) {
        unsafe {
            let channel_address = self.tx_channel_address;
            println!("DUMPING CHANNEL tx ({}) {:p}", text, channel_address);
            let channel_header = channel_address as *mut ChannelHeader;
            if (*channel_header).first_message_offset == 0 {
                println!("  EMPTY");
            }
            else {
                let mut index = (*channel_header).first_message_offset;
                let mut iter = channel_address.offset((*channel_header).first_message_offset as isize) as *const ChannelMessageHeader;
                'messages: loop {
                    println!("  {}: prev: {}, next: {}, size: {}", index, (*iter).previous_message_offset, (*iter).next_message_offset, (*iter).message_length);
                    if (*iter).next_message_offset == 0 {
                        break 'messages;
                    }
                    index = (*iter).next_message_offset;
                    iter = channel_address.offset((*iter).next_message_offset as isize) as *const ChannelMessageHeader;
                }
            }
        }
    }

    pub fn number_of_messages_available(&self) -> usize {
        0
    }

    pub fn prepare_message(&mut self, message_id: u64, replace_pending: bool) -> (u64, *mut ChannelMessageHeader) {
        self.dump_tx("prepare_message BEFORE");
        unsafe {
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get("prepare_message", self.tx_channel_address);
            let call_id = self.call_id;
            self.call_id += 1;
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            assert!(!(*channel_header).is_writing);
            let mut message: *mut ChannelMessageHeader;
            if (*channel_header).first_message_offset == 0 {
                (*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();
                (*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();
                message = self.tx_channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;
                (*message).previous_message_offset = 0;
            }
            else {
                let last_message_offset = (*channel_header).last_message_offset;
                let last_message = self.tx_channel_address.offset(last_message_offset as isize) as *mut ChannelMessageHeader;
                let new_last_message_offset = last_message_offset + (*last_message).message_length;
                (*last_message).next_message_offset = new_last_message_offset;
                (*channel_header).last_message_offset = new_last_message_offset;
                message = self.tx_channel_address.offset(new_last_message_offset as isize) as *mut ChannelMessageHeader;
                (*message).previous_message_offset = last_message_offset;
            }
            (*channel_header).is_writing = true;
            (*message).message_magic = ChannelMessageHeader::MAGIC;
            (*message).message_id = message_id;
            (*message).call_id = call_id;
            (*message).replace_pending = replace_pending;
            (*message).message_length = 0;
            (*message).next_message_offset = 0;
            (*message).is_writing = true;
            (*message).pending_unlink = false;
            self.dump_tx("prepare_message AFTER");
            (call_id, message)
        }
    }

    pub fn commit_message(&self, message_payload_size: usize) {
        self.dump_tx("commit_message BEFORE");
        unsafe {
            let channel_header = self.tx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get("commit_message", self.tx_channel_address);
            let last_message = self.tx_channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            assert!((*channel_header).is_writing);
            #[cfg(debug)]
            assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);
            (*channel_header).is_writing = false;
            (*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;
            (*last_message).is_writing = false;
            self.dump_tx("commit_message AFTER");
        }
    }

    pub fn find_specific_message(&self, call_id: u64) -> Option<*mut ChannelMessageHeader> {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let lock = ChannelLock::get("find_specific_message", self.rx_channel_address);
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            if (*channel_header).first_message_offset == 0 {
                None
            }
            else {
                let first_message = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
                #[cfg(debug)]
                assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
                let mut iter = first_message;
                while (*iter).call_id != call_id && (*iter).next_message_offset != 0 && !(*iter).is_writing {
                    iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                }
                if (*iter).call_id == call_id {
                    assert!(!(*iter).pending_unlink);
                    (*iter).pending_unlink = true;
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
            let lock = ChannelLock::get("find_message", self.rx_channel_address);
            #[cfg(debug)]
            assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
            if (*channel_header).first_message_offset == 0 {
                None
            }
            else {
                let first_message = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;
                #[cfg(debug)]
                assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);
                if !(*first_message).replace_pending {
                    (*first_message).pending_unlink = true;
                    Some(first_message)
                }
                else {
                    let mut last_of_kind = first_message;
                    let mut iter = first_message;
                    while (*iter).next_message_offset != 0 && !(*iter).is_writing {
                        iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                        if (*iter).message_id == (*first_message).message_id && !(*iter).pending_unlink {
                            last_of_kind = iter;
                        }
                    }
                    let mut iter = first_message;
                    while (*iter).next_message_offset != 0 && iter != last_of_kind && !(*iter).is_writing {
                        let next_message_offset = (*iter).next_message_offset;
                        if (*iter).message_id == (*first_message).message_id && !(*iter).pending_unlink {
                            self.unlink_message(iter, true);
                        }
                        iter = self.rx_channel_address.offset(next_message_offset as isize) as *mut ChannelMessageHeader;
                    }
                    (*last_of_kind).pending_unlink = true;
                    Some(last_of_kind)
                }
            }
        }
    }

    pub fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool) {
        unsafe {
            let channel_header = self.rx_channel_address as *mut ChannelHeader;
            let lock = if lock_held { None } else { Some(ChannelLock::get("unlink_message", self.rx_channel_address)) };
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
        }
    }
}


