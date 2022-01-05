use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::sync::Arc;
use std::sync::Mutex;

pub const BOGUS_AUTO_BOTH_MIXED_CLIENT_TO_SERVER_MESSAGE: u64 = 5;

pub fn call_vec(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::BothMixedArgumentsEnum>) -> Result<crate::BothMixedMixedResultIterator, Error> {
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

pub fn add(channel_reference: Arc<Mutex<Channel>>, object: crate::BothMixedArgumentsEnum) {
    let mut channel = channel_reference.lock().unwrap();
    channel.add_object(crate::BOGUS_AUTO_BOTH_MIXED_ARGUMENTS_ENUM_OBJECT_ID, object);
}

pub fn call(channel_reference: Arc<Mutex<Channel>>) -> Result<crate::BothMixedMixedResultIterator, Error> {
    let channel = channel_reference.lock().unwrap();
    let result = channel.call_sync(BOGUS_AUTO_BOTH_MIXED_CLIENT_TO_SERVER_MESSAGE, false, 1000);
    drop(channel);
    match result {
        Ok(()) => {
            Ok(crate::BothMixedMixedResultIterator::new(channel_reference.clone()))
        },
        Err(error) => {
            Err(error)
        }
    }
}

pub fn handle(handler: &mut Box<dyn BogusAutoServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let iterator = BothMixedMixedArgumentsIterator::new(channel_reference.clone());
    let result = handler.both_mixed(iterator);

    channel.start();
    channel.send(Channel::to_reply(BOGUS_AUTO_BOTH_MIXED_CLIENT_TO_SERVER_MESSAGE, false));
}
