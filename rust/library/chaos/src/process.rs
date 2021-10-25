use crate::{ syscalls, channel, error::Error, handle::Handle, channel::Channel, action::Action };

use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref ON_CONNECT: Mutex<HashMap<u64, fn(Handle, Channel) -> ()>> = {
        Mutex::new(HashMap::new())
    };
    static ref ON_APPLICATION_START: Mutex<Option<fn() -> ()>> = {
        Mutex::new(None)
    };
}

pub fn wrap(name: &str, main: fn() -> ()) -> () {
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

pub fn on_application_start(handler: Option<fn() -> ()>) {
    *ON_APPLICATION_START.lock().unwrap() = handler;
}

pub fn on_connect(handle: Handle, handler: Option<fn(Handle, Channel) -> ()>) {
    match handler {
        Some(f) => {
            ON_CONNECT.lock().unwrap().insert(handle.id, f);
        },
        None => {
            ON_CONNECT.lock().unwrap().remove(&handle.id);
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
                        match ON_CONNECT.lock().unwrap().get(&target_handle.id) {
                            Some(f) => {
                                let channel = Channel::new(argument_handle);
                                f(target_handle, channel);
                            },
                            None => {}
                        }
                    },
                    Action::Signal => {
                        channel::signal(target_handle, parameter);
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
    std::process::abort();
}