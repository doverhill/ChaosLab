use library_chaos::{ Error, Channel, ChannelObject };
use std::mem;
use std::iter::Iterator;
use std::sync::{ Arc, Mutex };
use crate::server::BogusServerImplementation;
use crate::types::FileInfo;

pub const BOGUS_GET_FILES_ARGUMENTS_OBJECT_ID: usize = 3;
#[derive(Default)]
pub struct GetFilesArguments {
    pub path: String
}

impl GetFilesArguments {
    pub fn new(path: &str) -> Self {
        GetFilesArguments {
            path: path.to_string()
        }
    }
}

impl ChannelObject for GetFilesArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {
        // write fixed size fields

        // write dynamic size fields
        let path_length = self.path.len();
        *(pointer as *mut usize) = path_length;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.path.as_ptr(), pointer, path_length);

        mem::size_of::<usize>() + path_length
    }

    unsafe fn from_channel(pointer: *const u8) -> Self {
        let mut object = GetFilesArguments::default();

        // read fixed size fields

        // read dynamic size fields
        let path_length = *(pointer as *const usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        object.path = std::str::from_utf8_unchecked(std::slice::from_raw_parts(pointer as *const u8, path_length)).to_owned();

        object
    }
}

pub struct GetFilesCallIterator {
    channel_reference: Arc<Mutex<Channel>>,
    index: usize,
    item_count: usize
}

impl GetFilesCallIterator {
    pub fn new(channel_reference: Arc<Mutex<Channel>>) -> Self {
        let channel = channel_reference.lock().unwrap();
        let item_count = channel.get_object_count();
        drop(channel);

        GetFilesCallIterator { 
            channel_reference: channel_reference.clone(), 
            index: 0,
            item_count: item_count
        }
    }

    pub fn item_count(&self) -> usize {
        self.item_count
    }
}

impl Iterator for GetFilesCallIterator {
    type Item = FileInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let channel = self.channel_reference.lock().unwrap();
            self.index += 1;
            match channel.get_object::<FileInfo>(self.index - 1, crate::types::BOGUS_TYPE_FILE_INFO_OBJECT_ID) {
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

pub fn call(channel_reference: Arc<Mutex<Channel>>, path: &str) -> Result<GetFilesCallIterator, Error> {
    let mut channel = channel_reference.lock().unwrap();
    channel.start();
    let arguments = GetFilesArguments::new(path);
    channel.add_object(BOGUS_GET_FILES_ARGUMENTS_OBJECT_ID, arguments);
    
    let result = channel.call_sync(crate::client::BOGUS_GET_FILES_CLIENT_MESSAGE, false, 1000);
    drop(channel);

    match result {
        Ok(()) => {
            Ok(GetFilesCallIterator::new(channel_reference.clone()))
        },
        Err(error) => {
            Err(error)
        }
    }
}

pub fn handle(handler: &mut Box<dyn BogusServerImplementation + Send>, channel_reference: Arc<Mutex<Channel>>) {
    let mut channel = channel_reference.lock().unwrap();

    let arguments = match channel.get_object::<GetFilesArguments>(0, BOGUS_GET_FILES_ARGUMENTS_OBJECT_ID) {
        Ok(arguments) => {
            arguments
        },
        Err(error) => {
            panic!("Failed to get arguments: {:?}", error);
        }
    };

    let result = handler.get_files(&arguments.path);

    // FIXME detect when channel is full and send partial result using has_more flag
    channel.start();
    for object in result {
        channel.add_object(crate::types::BOGUS_TYPE_FILE_INFO_OBJECT_ID, object);
    }

    channel.send(Channel::to_reply(crate::client::BOGUS_GET_FILES_CLIENT_MESSAGE, false));
}