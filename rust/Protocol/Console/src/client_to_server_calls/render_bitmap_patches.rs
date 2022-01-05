use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const CONSOLE_RENDER_BITMAP_PATCHES_CLIENT_TO_SERVER_MESSAGE: u64 = 5;

pub fn call(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::BitmapPatch>) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    for object in objects {
        channel.add_object(crate::CONSOLE_BITMAP_PATCH_OBJECT_ID, object);
    }
    channel.call_sync(CONSOLE_RENDER_BITMAP_PATCHES_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::ConsoleServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let iterator = crate::RenderBitmapPatchesBitmapPatchIterator::new(channel_reference.clone());
    let result = handler.render_bitmap_patches(iterator);
    let mut channel = channel_reference.lock().unwrap();

    channel.start();
    channel.send(Channel::to_reply(CONSOLE_RENDER_BITMAP_PATCHES_CLIENT_TO_SERVER_MESSAGE, false));
}
