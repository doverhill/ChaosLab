use crate::{ error::Error, syscalls, action::Action, service::Service, channel::Channel };

pub struct Process {}

impl Process {
    pub fn set_info(process_name: &str) -> Result<(), Error> {
        syscalls::process_set_info(process_name)
    }

    pub fn emit_debug(information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(
            syscalls::EmitType::Debug,
            Error::None,
            Some(information_text),
        )
    }

    pub fn emit_information(information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(
            syscalls::EmitType::Information,
            Error::None,
            Some(information_text),
        )
    }

    pub fn emit_warning(information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(
            syscalls::EmitType::Warning,
            Error::None,
            Some(information_text),
        )
    }

    pub fn emit_error(error: Error, information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(syscalls::EmitType::Error, error, Some(information_text))
    }

    pub fn run() -> Error {
        // this is the main event loop of an application
        loop {
            match syscalls::event_wait(None, None, None, -1) {
                Ok((target_handle, argument_handle, action, parameter)) => {
                    match action {
                        Action::ServiceConnected => {
                            Service::connected(target_handle, argument_handle);
                        },
                        Action::ChannelMessaged => {
                            Channel::messaged(target_handle, parameter);
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

    pub fn end() -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
