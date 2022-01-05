use library_chaos::{ Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_BOTH_MIXED_RESULT_ENUM_OBJECT_ID: usize = 11;

pub enum BothMixedResultEnum {
    FileInfo(crate::FileInfo),
    Component(crate::Component)
}

impl ChannelObject for BothMixedResultEnum {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        match self {
            Self::FileInfo(object) => {
                *(pointer as *mut usize) = crate::BOGUS_AUTO_FILE_INFO_OBJECT_ID;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let size = object.write_to_channel(pointer);
                mem::size_of::<usize>() + size
            },
            Self::Component(object) => {
                *(pointer as *mut usize) = crate::BOGUS_AUTO_COMPONENT_OBJECT_ID;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let size = object.write_to_channel(pointer);
                mem::size_of::<usize>() + size
            }
        }
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let kind = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        match kind {
            crate::BOGUS_AUTO_FILE_INFO_OBJECT_ID => {
                Self::FileInfo(crate::FileInfo::from_channel(pointer))
            },
            crate::BOGUS_AUTO_COMPONENT_OBJECT_ID => {
                Self::Component(crate::Component::from_channel(pointer))
            },
            _ => {
                panic!("Received unexpected value for BothMixedResultEnum");
            }
        }
    }
}

pub struct BothMixedMixedResultIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl BothMixedMixedResultIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> Self {
        let channel = channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        BothMixedMixedResultIterator {
            channel_reference: channel_reference.clone(),
            index: 0,
            item_count: item_count
        }
    }
}

impl Iterator for BothMixedMixedResultIterator {
    type Item = crate::BothMixedResultEnum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let channel = self.channel_reference.lock().unwrap();
            self.index += 1;
            match channel.get_object::<crate::BothMixedResultEnum>(self.index - 1, crate::BOGUS_AUTO_BOTH_MIXED_RESULT_ENUM_OBJECT_ID) {
                Ok(object) => {
                    Some(object)
                },
                Err(_) => {
                    None
                }
            }
        }
        else {
            None
        }
    }
}

