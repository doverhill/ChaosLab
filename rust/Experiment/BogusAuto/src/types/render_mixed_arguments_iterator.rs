use library_chaos::{ Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_RENDER_ARGUMENTS_ENUM_OBJECT_ID: usize = 8;

pub enum RenderArgumentsEnum {
    Window(crate::Window),
    Button(crate::Button)
}

impl ChannelObject for RenderArgumentsEnum {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        match self {
            Self::Window(object) => {
                *(pointer as *mut usize) = crate::BOGUS_AUTO_WINDOW_OBJECT_ID;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let size = object.write_to_channel(pointer);
                mem::size_of::<usize>() + size
            },
            Self::Button(object) => {
                *(pointer as *mut usize) = crate::BOGUS_AUTO_BUTTON_OBJECT_ID;
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
            crate::BOGUS_AUTO_WINDOW_OBJECT_ID => {
                Self::Window(crate::Window::from_channel(pointer))
            },
            crate::BOGUS_AUTO_BUTTON_OBJECT_ID => {
                Self::Button(crate::Button::from_channel(pointer))
            },
            _ => {
                panic!("Received unexpected value for RenderArgumentsEnum");
            }
        }
    }
}

pub struct RenderMixedArgumentsIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl RenderMixedArgumentsIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> Self {
        let channel = channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        RenderMixedArgumentsIterator {
            channel_reference: channel_reference.clone(),
            index: 0,
            item_count: item_count
        }
    }
}

impl Iterator for RenderMixedArgumentsIterator {
    type Item = crate::RenderArgumentsEnum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let channel = self.channel_reference.lock().unwrap();
            self.index += 1;
            match channel.get_object::<crate::RenderArgumentsEnum>(self.index - 1, crate::BOGUS_AUTO_RENDER_ARGUMENTS_ENUM_OBJECT_ID) {
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

