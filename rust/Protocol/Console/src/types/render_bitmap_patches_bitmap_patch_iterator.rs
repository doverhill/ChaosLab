use library_chaos::Channel;
use core::{ mem, ptr, str, slice };
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::Mutex;

pub struct RenderBitmapPatchesBitmapPatchIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl RenderBitmapPatchesBitmapPatchIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> Self {
        let channel = channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        RenderBitmapPatchesBitmapPatchIterator {
            channel_reference: channel_reference.clone(),
            index: 0,
            item_count: item_count
        }
    }
}

impl Iterator for RenderBitmapPatchesBitmapPatchIterator {
    type Item = crate::BitmapPatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let channel = self.channel_reference.lock().unwrap();
            self.index += 1;
            match channel.get_object::<crate::BitmapPatch>(self.index - 1, crate::CONSOLE_BITMAP_PATCH_OBJECT_ID) {
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

