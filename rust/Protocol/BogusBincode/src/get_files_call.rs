use library_chaos::{ Error, Channel, ChannelObject };
use std::mem;
use std::iter::Iterator;
use std::sync::{ Arc, Mutex };
use crate::server::BogusServerImplementation;
use crate::types::FileInfo;
use serde::{ Serialize, Deserialize };

pub const BOGUS_GET_FILES_ARGUMENTS_OBJECT_ID: usize = 3;
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GetFilesArguments {
    pub path: String
}

// impl ChannelObject for GetFilesArguments {
//     unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
//         // write fixed size fields

//         // write dynamic size fields
//         let path_length = self.path.len();
//         *(pointer as *mut usize) = path_length;
//         let pointer = pointer.offset(mem::size_of::<usize>() as isize);
//         core::ptr::copy(self.path.as_ptr(), pointer, path_length);

//         mem::size_of::<usize>() + path_length
//     }

//     unsafe fn from_channel(pointer: *const u8) -> Self {
//         let mut object = GetFilesArguments::default();

//         // read fixed size fields

//         // read dynamic size fields
//         let path_length = *(pointer as *const usize);
//         let pointer = pointer.offset(mem::size_of::<usize>() as isize);
//         object.path = String::from_raw_parts(pointer as *mut u8, path_length, path_length);

//         object
//     }
// }

pub struct ReturnIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl ReturnIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> ReturnIterator {
        let channel = &*channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        ReturnIterator { 
            channel_reference: channel_reference.clone(), 
            index: 0,
            item_count: item_count
        }
    }
}

impl Iterator for ReturnIterator {
    type Item = FileInfo;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn call(channel_reference: Arc<Mutex<Channel>>, path: &str) -> Result<ReturnIterator, Error> {
    let channel = &mut *channel_reference.lock().unwrap();
    channel.start();
    let arguments = GetFilesArguments {
        path: path.to_string()
    };
    channel.add_object(BOGUS_GET_FILES_ARGUMENTS_OBJECT_ID, arguments);
    
    match channel.call_sync(crate::client::BOGUS_GET_FILES_CLIENT_MESSAGE, false, 1000) {
        Ok(()) => {
            Ok(ReturnIterator::new(channel_reference.clone()))
        },
        Err(error) => {
            Err(error)
        }
    }
}

pub fn handle(handler: &mut Box<dyn BogusServerImplementation + Send>, channel: &mut Channel) {
    let arguments = channel.get_object::<GetFilesArguments>(0);
    let result = handler.get_files(&arguments.path);

    // FIXME detect when channel is full and send partial result using has_more flag
    channel.start();
    for object in result {
        channel.add_object(crate::types::BOGUS_TYPE_FILE_INFO_OBJECT_ID, object);
    }
    channel.send(Channel::to_reply(crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE, false));
}