use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE: u64 = 3;

pub fn call(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::RenderArgumentsEnum>) -> Result<(), Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    for object in objects {
        channel.add_object(crate::BOGUS_AUTO_RENDER_ARGUMENTS_ENUM_OBJECT_ID, object);
    }
    channel.call_sync(BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn crate::BogusAutoServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let iterator = crate::RenderMixedArgumentsIterator::new(channel_reference.clone());
    let result = handler.render(iterator);
    let mut channel = channel_reference.lock().unwrap();

    channel.start();
    channel.send(Channel::to_reply(BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE, false));
}
