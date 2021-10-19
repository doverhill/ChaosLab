use crate::error::Error;
use crate::handle::Handle;
use crate::syscalls;

use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref ON_CONNECT: Mutex<HashMap<u64, fn(Handle) -> ()>> = {
        Mutex::new(HashMap::new())
    };
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

pub fn on_connect(handle: Handle, handler: Option<fn(Handle) -> ()>) {
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
            Ok((handle, action)) => {
                emit_debug(&format!("Got action {:?} on handle {}", action, handle));
                match ON_CONNECT.lock().unwrap().get(&handle.id) {
                    Some(f) => {
                        f(handle);
                    },
                    None => {}
                }
            },
            Err(error) => {
                return error;
            }
        }
    }
}

pub fn end() -> () {

}