use crate::{ syscalls, Error, Action, ServiceCollection, ChannelCollection, Event };

pub struct Process {
    name: String,
    pub services: ServiceCollection,
    pub channels: ChannelCollection,
}

impl Process {
    pub fn new(name: &str) -> Result<Process, Error> {
        syscalls::process_set_info(name);

        Ok(Process {
            name: name.to_string(),
            services: ServiceCollection::new(),
            channels: ChannelCollection::new()
        })
    }

    pub fn emit_debug(&self, information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(syscalls::EmitType::Debug, &Error::None, Some(information_text))
    }

    pub fn emit_information(information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(syscalls::EmitType::Information, &Error::None, Some(information_text))
    }

    pub fn emit_warning(information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(syscalls::EmitType::Warning, &Error::None, Some(information_text))
    }

    pub fn emit_error(error: &Error, information_text: &str) -> Result<(), Error> {
        syscalls::process_emit(syscalls::EmitType::Error, error, Some(information_text))
    }

    // pub fn run(&self) -> Error {
    //     // this is the main event loop of an application
    //     loop {
    //         match syscalls::event_wait(None, None, None, -1) {
    //             Ok((target_handle, argument_handle, action, parameter)) => {
    //                 match action {
    //                     Action::ServiceConnected => {
    //                         Service::connected(target_handle, argument_handle);
    //                     },
    //                     Action::ChannelMessaged => {
    //                         Channel::messaged(target_handle, parameter);
    //                     },
    //                     _ => {}
    //                 }
    //             },
    //             Err(error) => {
    //                 return error;
    //             }
    //         }
    //     }
    // }

    pub fn event_wait(&self) -> Result<Event, Error> {
        match syscalls::event_wait(handle, action, message, timeout_milliseconds)
    }

    pub fn end(&self) -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
