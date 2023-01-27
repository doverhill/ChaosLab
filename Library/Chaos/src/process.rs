use crate::{ syscalls, StormError, StormAction, ServiceCollection, ChannelCollection, StormEvent };

pub struct StormProcess {
    name: String,
    pub services: ServiceCollection,
    pub channels: ChannelCollection,
}

impl Drop for StormProcess {
    fn drop(&mut self) {
        self.end();
    }
}

impl StormProcess {
    pub fn new(name: &str) -> Result<Self, StormError> {
        syscalls::process_set_info(name);

        Ok(Self {
            name: name.to_string(),
            services: ServiceCollection::new(),
            channels: ChannelCollection::new()
        })
    }

    pub fn emit_debug(&self, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(syscalls::EmitType::Debug, StormError::None, Some(information_text))
    }

    pub fn emit_information(information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(syscalls::EmitType::Information, StormError::None, Some(information_text))
    }

    pub fn emit_warning(information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(syscalls::EmitType::Warning, StormError::None, Some(information_text))
    }

    pub fn emit_error(error: StormError, information_text: &str) -> Result<(), StormError> {
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

    // pub fn event_wait(&self) -> Result<Event, Error> {
    //     match syscalls::event_wait(handle, action, message, timeout_milliseconds)
    // }

    pub fn end(&self) -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
