use crate::{ syscalls, channel::Channel, handle::Handle, error::Error, action::Action, call::Call };
use std::sync::Mutex;
use std::collections::{ HashMap, HashSet };

struct ChannelData {
    channel: Channel,
    call_done_handlers: HashMap<u64, fn(*mut u8) -> ()>
}

lazy_static! {
    static ref HANDLES: Mutex<HashSet<u64>> = {
        Mutex::new(HashSet::new())
    };

    // channel handle id -> Channel
    static ref CHANNELS: Mutex<HashMap<u64, ChannelData>> = {
        Mutex::new(HashMap::new())
    };

    // service handle id -> Handler(service_handle_id, new_channel)
    static ref CONNECT_HANDLERS: Mutex<HashMap<u64, fn(Handle, &Channel) -> ()>> = {
        Mutex::new(HashMap::new())
    };

    // channel handle id -> Handler(channel, signal_number)
    static ref SIGNAL_HANDLERS: Mutex<HashMap<u64, fn(&Channel, u64) -> ()>> = {
        Mutex::new(HashMap::new())
    };
}

pub fn register_channel(channel: Channel) {
    let channels = &mut *CHANNELS.lock().unwrap();
    let channel_handle_id = channel.channel_handle.id;
    let channel_data = ChannelData {
        channel: channel,
        call_done_handlers: HashMap::new()
    };
    channels.insert(channel_handle_id, channel_data);
}

pub fn get_channel_pointer(channel_handle: Handle) -> Option<*mut u8> {
    let channels = &mut *CHANNELS.lock().unwrap();
    match channels.get(&channel_handle.id) {
        Some (channel_data) => {
            Some(channel_data.channel.map_pointer)
        },
        None => { None }
    }
}

pub fn channel_interface(channel_handle: Handle, function: u64) -> Call {
    Call {
        channel_handle: channel_handle,
        function: function
    }
}

pub fn wrap(name: &str, main: fn() -> ()) {
    set_info(name);
    main();    
    syscalls::cleanup();
}

pub fn set_info(process_name: &str) -> Option<Error> {
    syscalls::process_set_info(process_name)
}

pub fn emit_debug(information_text: &str) -> Option<Error> {
    syscalls::process_emit(syscalls::EmitType::Debug, Error::None, Some(information_text))
}

pub fn emit_information(information_text: &str) -> Option<Error> {
    syscalls::process_emit(syscalls::EmitType::Information, Error::None, Some(information_text))
}

pub fn emit_warning(information_text: &str) -> Option<Error> {
    syscalls::process_emit(syscalls::EmitType::Warning, Error::None, Some(information_text))
}

pub fn emit_error(error: Error, information_text: &str) -> Option<Error> {
    syscalls::process_emit(syscalls::EmitType::Error, error, Some(information_text))
}

pub fn on_connect(service_handle: Handle, handler: Option<fn(Handle, &Channel) -> ()>) {
    let connect_handlers = &mut *CONNECT_HANDLERS.lock().unwrap();

    match handler {
        Some(f) => {
            connect_handlers.insert(service_handle.id, f);
        },
        None => {
            connect_handlers.remove(&service_handle.id);
        }
    }
}

pub fn connected(service_handle: Handle, argument_handle: Handle) {
    let connect_handlers = &mut *CONNECT_HANDLERS.lock().unwrap();
    let channels = &mut *CHANNELS.lock().unwrap();

    match connect_handlers.get(&service_handle.id) {
        Some(f) => {
            let channel = Channel::new(argument_handle);
            f(service_handle, &channel);
            let channel_data = ChannelData {
                channel: channel,
                call_done_handlers: HashMap::new()
            };
            channels.insert(argument_handle.id, channel_data);
        },
        None => {}
    }
}

pub fn on_signal(channel: &Channel, handler: Option<fn(&Channel, u64) -> ()>) {
    let signal_handlers = &mut *SIGNAL_HANDLERS.lock().unwrap();

    match handler {
        Some(f) => {
            signal_handlers.insert(channel.channel_handle.id, f);
        },
        None => {
            signal_handlers.remove(&channel.channel_handle.id);
        }
    }
}

pub fn signalled(channel_handle: Handle, signal: u64)
{
    let signal_handlers = &mut *SIGNAL_HANDLERS.lock().unwrap();
    let channels = &mut *CHANNELS.lock().unwrap();

    match channels.get_mut(&channel_handle.id) {
        Some(channel_data) => {
            // see if there is a signal handler for this channel
            match signal_handlers.get(&channel_handle.id) {
                Some(f) => {
                    f(&channel_data.channel, signal);
                },
                None => {}
            }

            // see if there is a call done handler for this channel and signal
            match channel_data.call_done_handlers.get(&signal) {
                Some(call_done_handler) => {
                    call_done_handler(channel_data.channel.map_pointer);
                    channel_data.call_done_handlers.remove(&signal);
                },
                None => {}
            }
        },
        None => {}
    }
}

pub fn on_call_done(channel_handle: Handle, signal: u64, handler: fn(*mut u8) -> ()) {
    let channels = &mut *CHANNELS.lock().unwrap();
    match channels.get_mut(&channel_handle.id) {
        Some (channel_data) => {
            channel_data.call_done_handlers.insert(signal, handler);
        },
        None => {
            panic!("Tried to add call done handler on channel with handle {} which does not exist", channel_handle);
        }
    }
}

pub fn run() -> Error {
    // this is the main event loop of an application
    loop {
        let result = syscalls::event_wait(-1);
        match result {
            Ok((target_handle, argument_handle, action, parameter)) => {
                match action {
                    Action::Connect => {
                        connected(target_handle, argument_handle);
                    },
                    Action::Signal => {
                        signalled(target_handle, parameter);
                    },
                    _ => {}
                }

            },
            Err(error) => {
                return error;
            }
        }
    }
}

pub fn end() -> () {
    syscalls::process_destroy();
    syscalls::cleanup();
    std::process::exit(0);
}