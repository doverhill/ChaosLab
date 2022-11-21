use crate::{ syscalls, Handle, Process, Error, Action };
use std::fmt;
use ::winapi::{
    shared::ntdef::NULL,
    um::{
        handleapi::{ CloseHandle, INVALID_HANDLE_VALUE },
        memoryapi::{ CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS },
        winnt::{ HANDLE, PAGE_READWRITE }
    }
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::mem;
use std::slice;

use std::sync::atomic::AtomicU64;

pub struct ChannelHeader {
    lock: AtomicU64,
    channel_magic: u64,
    protocol_name_low: u64,
    protocol_name_high: u64,
    message_count: u64,
    write_offset: u64,
    read_offset: u64
}

pub struct ChannelMessageHeader {
    message_magic: u64,
    message_id: u64,
    message_flags: u64,
    message_length: u64,
    object_count: u64
}

pub struct ChannelMessageObjectHeader {
    object_id: u64,
    object_length: u64
}

lazy_static! {
    static ref CHANNELS: Mutex<HashMap<Handle, Arc<Mutex<Channel>>>> = {
        Mutex::new(HashMap::new())
    };
}

pub type Message = u64;
pub type ComposedMessage = u64;

pub trait ChannelObject {
    unsafe fn write_to_channel(self, pointer: *mut u8) -> usize;
    unsafe fn from_channel(pointer: *const u8) -> Self;
}

pub struct Channel {
    pub handle: Handle,
    map_handle: HANDLE,
    map_pointer: *mut u8,
    map_capacity: usize,
    body_pointer: *mut u8,
    allocation_pointer: *mut u8,
    message_handler: Option<fn(Arc<Mutex<Channel>>, u64) -> ()>
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}

impl Drop for Channel {
    fn drop(&mut self) {
        println!("dropping channel");
        if self.map_pointer as *mut _ != NULL {
            unsafe { UnmapViewOfFile(self.map_pointer as *mut _) };
        }

        if self.map_handle as *mut _ != NULL {
            unsafe { CloseHandle(self.map_handle) };
        }

        syscalls::channel_destroy(self.handle).unwrap();
    }
}

// CHANNEL MEMORY LAYOUT:
// ----------------------
// HEADER:
//     usize ChannelInitialized (0 = No, 0x1337 = Yes)
//     usize ProtocolVersion
//     usize ReplyReadyFlag
//     usize ProtocolNameLength
//     u8[] ProtocolName
// BODY:
//     usize ObjectCount
//     [
//         usize ObjectId // created by IDL compiler for each type to be sent on the channel
//         usize ObjectLength
//         u8[] ObjectData
//     ]*

impl Channel {
    pub fn new(handle: Handle, size: usize) -> Arc<Mutex<Channel>> {
        let memory_name = &Channel::get_map_name(&handle);
        let (map_handle, map_pointer) = Channel::create_shared_memory(memory_name, size);
        map_handle.expect("Failed to create shared memory");

        let channel = Arc::new(Mutex::new(Channel {
            handle: handle,
            map_handle: map_handle.unwrap(),
            map_pointer: map_pointer,
            map_capacity: size,
            body_pointer: map_pointer,
            allocation_pointer: map_pointer,
            message_handler: None
        }));

        let mut channels = CHANNELS.lock().unwrap();
        channels.insert(handle, channel.clone());

        channel
    }

    fn create_shared_memory(name: &str, size: usize) -> (Option<HANDLE>, *mut u8) {
        let high_size: u32 = ((size & 0xFFFF_FFFF_0000_0000_usize) >> 32) as u32;
        let low_size: u32 = (size & 0xFFFF_FFFF_usize) as u32;
        let unique_id: Vec<u16> = OsStr::new(name).encode_wide().chain(once(0)).collect();

        let map_handle = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                null_mut(),
                PAGE_READWRITE,
                high_size,
                low_size,
                unique_id.as_ptr(),
            )
        };

        if map_handle == NULL {
            return (None, null_mut());
        }

        let map_pointer = unsafe { MapViewOfFile(map_handle, FILE_MAP_ALL_ACCESS, 0, 0, 0) } as _;

        (Some(map_handle), map_pointer)
    }

    fn get_map_name(handle: &Handle) -> String {
        return format!("Local\\__chaos_channel_{}", handle);
    }

    pub fn compose_message(message: Message, is_reply: bool, has_more: bool) -> ComposedMessage {
        message
            | if is_reply { 1 << 63 } else { 0 }
            | if has_more { 1 << 62 } else { 0 }
    }

    pub fn get_message(message: ComposedMessage) -> Message {
        message & !((1 << 63) | (1 << 62))
    }

    pub fn get_is_reply(message: ComposedMessage) -> bool {
        message & (1 << 63) != 0
    }

    pub fn get_has_more(message: ComposedMessage) -> bool {
        message & (1 << 62) != 0
    }

    pub fn to_reply(message: Message, has_more: bool) -> ComposedMessage {
        Self::compose_message(message, true, has_more)
    }

    pub fn from_reply(message: ComposedMessage) -> Message {
        let is_reply = Self::get_is_reply(message);
        if !is_reply {
            panic!("Tried to get reply from composed message which is not a reply");
        }
        
        Self::get_message(message)
    }

    pub fn on_message(&mut self, handler: fn(Arc<Mutex<Channel>>, u64) -> ()) -> Result<(), Error> {
        match self.message_handler {
            Some(_) => {
                Err(Error::AlreadyExists)
            },
            None => {
                self.message_handler = Some(handler);
                Ok(())
            }
        }
    }

    pub(crate) fn messaged(handle: Handle, message: u64) {
        Process::emit_debug(&format!("Channel {} got message {}", handle, message)).unwrap();

        let channels = CHANNELS.lock().unwrap();
        if let Some(channel_wrap) = channels.get(&handle) {
            let channel = channel_wrap.lock().unwrap();
            if let Some(handler) = channel.message_handler {
                drop(channel); // release mutex
                handler(channel_wrap.clone(), message);
            }
        }
    }

    pub fn initialize(&mut self, protocol_name: &str, protocol_version: usize) {
        if self.is_initialized() {
            panic!("Tried to initialize already initialized channel");
        }

        unsafe {
            let pointer = self.map_pointer;

            // initialized
            *(pointer as *mut usize) = 0x1337_1337_1337_1337;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);

            // version
            *(pointer as *mut usize) = protocol_version;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);

            // reply ready
            *(pointer as *mut usize) = 0;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);

            // protocol name length
            *(pointer as *mut usize) = protocol_name.len();
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);

            // protocol name
            core::ptr::copy(protocol_name.as_ptr(), pointer, protocol_name.len());
            let pointer = pointer.offset(protocol_name.len() as isize);

            self.body_pointer = pointer;
        }
    }

    pub fn is_initialized(&self) -> bool {
        unsafe {
            *(self.map_pointer as *mut usize) == 0x1337_1337_1337_1337
        }
    }

    pub fn get_protocol_version(&self) -> usize {
        if !self.is_initialized() {
            panic!("Tried to get protocol version of uninitialized channel");
        }

        unsafe {
            // skip initialized field
            let pointer = self.map_pointer.offset(mem::size_of::<usize>() as isize);
            *(pointer as *const usize)
        }
    }

    pub fn get_protocol_name(&self) -> &str {
        if !self.is_initialized() {
            panic!("Tried to get protocol name of uninitialized channel");
        }

        unsafe {
            // skip initialized field, version and ready flag
            let pointer = self.map_pointer.offset(3 * mem::size_of::<usize>() as isize);
            let length = *(pointer as *const usize);
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::str::from_utf8_unchecked(slice::from_raw_parts(pointer, length))
        }
    }

    pub fn get_object_count(&self) -> usize {
        if !self.is_initialized() {
            panic!("Tried to get object count of uninitialized channel");
        }

        unsafe {
            // skip initialized field, version and ready flag
            let pointer = self.map_pointer.offset(3 * mem::size_of::<usize>() as isize);
            let length = *(pointer as *const usize);
            let pointer = pointer.offset((mem::size_of::<usize>() + length) as isize);
            *(pointer as *const usize)
       }
    }

    pub unsafe fn get_object_wrapper_pointer(&self, index: usize) -> *const u8 {
        // skip initialized field, version and ready flag
        let pointer = self.map_pointer.offset(3 * mem::size_of::<usize>() as isize);
        let length = *(pointer as *const usize);

        // skip protocol name length and actual string
        let pointer = pointer.offset((mem::size_of::<usize>() + length) as isize);

        // get object count
        let object_count = *(pointer as *const usize);
        if index >= object_count {
            panic!("Tried to get object {}, but there are only {} objects", index, object_count);
        }
        let mut pointer = pointer.offset((mem::size_of::<usize>()) as isize);

        for _ in 0..index {
            // skip object id and get length
            pointer = pointer.offset(mem::size_of::<usize>() as isize);
            let object_length = *(pointer as *const usize);
            pointer = pointer.offset((mem::size_of::<usize>() + object_length) as isize);
        }

        pointer
    }

    pub unsafe fn get_object_pointer(&self, index: usize) -> *mut u8 {
        let pointer = self.get_object_wrapper_pointer(index);
        let pointer = pointer.offset(2 * mem::size_of::<usize>() as isize);
        pointer as *mut u8
    }

    pub fn get_object_id(&self, index: usize) -> usize {
        unsafe {
            let pointer = self.get_object_wrapper_pointer(index);
            *(pointer as *const usize)
        }
    }

    pub fn get_object_length(&self, index: usize) -> usize {
        unsafe {
            let pointer = self.get_object_wrapper_pointer(index);
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            *(pointer as *const usize)
        }
    }

    pub fn get_object<T : ChannelObject>(&self, index: usize, expected_id: usize) -> Result<T, Error> {
        unsafe {
            let count = self.get_object_count();
            if index >= count {
                println!("ERROR: Tried to get object at index {}, but there are only {} objects", index, count);
                return Err(Error::NotFound);
            }

            let id = self.get_object_id(index);
            if id != expected_id {
                println!("Expected object with id {} at index {}, but found object with id {}", expected_id, index, id);
                return Err(Error::NotFound);
            }

            let pointer = self.get_object_pointer(index);
            Ok(T::from_channel(pointer))
        }
    }

    pub fn send(&self, message: u64) {
        syscalls::channel_message(self.handle, message).unwrap();
    }

    pub fn start(&mut self) {
        if !self.is_initialized() {
            panic!("Tried to start sending on uninitialized channel");
        }

        // set up body pointer, initial object count and allocation pointer
        unsafe {
            // skip initialized field, version and ready flag
            let pointer = self.map_pointer.offset(3 * mem::size_of::<usize>() as isize);
            let length = *(pointer as *const usize);
            let pointer = pointer.offset((mem::size_of::<usize>() + length) as isize);
            self.body_pointer = pointer;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            self.allocation_pointer = pointer;

            // initial object count is 0
            *(self.body_pointer as *mut usize) = 0;
        }
    }

    pub fn add_object<T : ChannelObject>(&mut self, object_id: usize, object: T) {
        if !self.is_initialized() {
            panic!("Tried to add object on uninitialized channel");
        }

        // FIXME: Use self.map_capacity and self.allocation_pointer to figure out remaining space. pass along to object.write_to_channel and make that function fallible if there is not enough room
        let _ = self.map_capacity;

        unsafe {
            let pointer = self.allocation_pointer;
            *(pointer as *mut usize) = object_id;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            let size_pointer = pointer;
            let pointer = pointer.offset(mem::size_of::<usize>() as isize);
            let size = object.write_to_channel(pointer);
            *(size_pointer as *mut usize) = size;
            self.allocation_pointer = self.allocation_pointer.offset((2 * mem::size_of::<usize>() + size) as isize);

            // increase object count
            *(self.body_pointer as *mut usize) = *(self.body_pointer as *const usize) + 1;
        }
    }

    pub fn call_sync(&self, message: u64, has_more: bool, timeout_milliseconds: i32) -> Result<(), Error> {
        syscalls::channel_message(self.handle, Self::compose_message(message, false, has_more))?;
        match syscalls::event_wait(Some(self.handle), Some(Action::ChannelMessaged), Some(Channel::to_reply(message, false)), timeout_milliseconds) {
            Ok((_, _, _, _)) => {
                Ok(())
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[CHANNEL: handle={}, map_handle={:?}, buffer={:p}]", self.handle, self.map_handle, self.map_pointer)
    }
}
