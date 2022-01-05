use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE: u64 = 3;

pub fn call_vec(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::RenderArgumentsEnum>) -> Result<(), Error> {
    start(channel_reference.clone());
    for object in objects {
        add(channel_reference.clone(), object);
    }
    call(channel_reference.clone())
}

pub fn start(channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
}

pub fn add(channel_reference: Arc<Mutex<Channel>>, object: crate::RenderArgumentsEnum) {
    let mut channel = channel_reference.lock().unwrap();
    channel.add_object(crate::BOGUS_AUTO_RENDER_ARGUMENTS_ENUM_OBJECT_ID, object);
}

pub fn call(channel_reference: Arc<Mutex<Channel>>) -> Result<(), Error> {
    let channel = channel_reference.lock().unwrap();
    channel.call_sync(BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE, false, 1000)
}

pub fn handle(handler: &mut Box<dyn BogusAutoServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let iterator = RenderMixedArgumentsIterator::new(channel_reference.clone());
    let result = handler.render(iterator);

    channel.start();
    channel.send(Channel::to_reply(BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE, false));
}
