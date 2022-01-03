use library_chaos::{ Error, Channel, ChannelObject };
use std::mem;
use std::iter::Iterator;
use std::sync::{ Arc, Mutex };
use crate::server::BogusServerImplementation;
use crate::types::*;

pub const BOGUS_RENDER_TYPE_ARGUMENTS_OBJECT_ID: usize = 9;
pub enum RenderTypeArguments {
    Window(Window),
    Button(Button)
}

impl ChannelObject for RenderTypeArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        match self {
            Self::Window(window) => {
                *(pointer as *mut usize) = BOGUS_TYPE_WINDOW_OBJECT_ID;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let size = window.write_to_channel(pointer);
                mem::size_of::<usize>() + size
            },
            Self::Button(button) => 
            {
                *(pointer as *mut usize) = BOGUS_TYPE_BUTTON_OBJECT_ID;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let size = button.write_to_channel(pointer);
                mem::size_of::<usize>() + size
            }
        }
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let kind = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        match kind {
            BOGUS_TYPE_WINDOW_OBJECT_ID => {
                Self::Window(Window::from_channel(pointer))
            },
            BOGUS_TYPE_BUTTON_OBJECT_ID => {
                Self::Button(Button::from_channel(pointer))
            },
            _ => {
                panic!("Received unexpected component kind {}", kind);
            }
        }
    }
}

pub struct RenderHandleIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl RenderHandleIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> RenderHandleIterator {
        let channel = channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        RenderHandleIterator { 
            channel_reference: channel_reference.clone(), 
            index: 0,
            item_count: item_count
        }
    }

    pub fn item_count(&self) -> usize {
        self.item_count
    }
}

impl Iterator for RenderHandleIterator {
    type Item = RenderTypeArguments;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let channel = self.channel_reference.lock().unwrap();
            self.index += 1;
            match channel.get_object::<RenderTypeArguments>(self.index - 1, BOGUS_RENDER_TYPE_ARGUMENTS_OBJECT_ID) {
                Ok(object) => {
                    Some(object)
                },
                Err(error) => {
                    None
                }
            }
        }
        else {
            None
        }
    }
}

pub fn start(channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
}

pub fn add(channel_reference: Arc<Mutex<Channel>>, component: RenderTypeArguments) {
    let mut channel = channel_reference.lock().unwrap();
    channel.add_object(BOGUS_RENDER_TYPE_ARGUMENTS_OBJECT_ID, component);
}

pub fn call(channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    channel.call_sync(crate::client::BOGUS_RENDER_CLIENT_MESSAGE, false, 1000);
}

pub fn handle(handler: &mut Box<dyn BogusServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let iterator = RenderHandleIterator::new(channel_reference.clone());
    handler.render(iterator);;
    let mut channel = channel_reference.lock().unwrap();
    channel.send(Channel::to_reply(crate::client::BOGUS_RENDER_CLIENT_MESSAGE, false));
}