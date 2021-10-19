use crate::error::Error;
use crate::syscalls;

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

pub fn run() -> Error {
    // this is the main event loop of an application
    while (true) {
        let result = syscalls::event_wait();
        match result {
            Ok(handle, action) => {

            },
            Err(error) => {
                return error;
            }
        }
    }

    Error::None
}

pub fn end() -> () {

}