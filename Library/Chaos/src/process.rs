use crate::{syscalls, ChannelHandle, ServiceHandle, StormAction, StormError, StormEvent, channel::Channel, service::Service};

use std::collections::HashMap;
use uuid::Uuid;

pub struct StormProcess {
    name: String,
    services: HashMap<ServiceHandle, Service>,
    channels: HashMap<ChannelHandle, Channel>,
}

impl Drop for StormProcess {
    fn drop(&mut self) {
        self.end();
    }
}

impl StormProcess {
    pub fn new(name: &str) -> Result<Self, StormError> {
        syscalls::process_set_info(name)?;

        Ok(Self {
            name: name.to_string(),
            services: HashMap::new(),
            channels: HashMap::new(),
        })
    }

    pub fn create_service(
        &mut self,
        protocol_name: &str,
        vendor_name: &str,
        device_name: &str,
        device_id: Uuid,
    ) -> Result<ServiceHandle, StormError> {
        let handle = syscalls::service_create(protocol_name, vendor_name, device_name, device_id)?;
        self.services.insert(handle, Service::new());
        Ok(handle)
    }

    pub fn destroy_service(&mut self, handle: ServiceHandle) -> Result<(), StormError> {
        self.services.remove(&handle);
        syscalls::service_destroy(handle)
    }

    pub fn on_service_connected(
        &mut self,
        handle: ServiceHandle,
        handler: Option<Box<dyn Fn(ServiceHandle, ChannelHandle)>>,
    ) -> Result<(), StormError> {
        match self.services.get_mut(&handle) {
            Some(service) => {
                service.on_connected = handler;
                Ok(())
            },
            None => Err(StormError::NotFound)
        }
    }

    pub fn connect_to_service(
        &mut self,
        protocol_name: &str,
        vendor_name: Option<&str>,
        device_name: Option<&str>,
        device_id: Option<Uuid>,
    ) -> Result<ChannelHandle, StormError> {
        let handle = syscalls::service_connect(protocol_name, vendor_name, device_name, device_id)?;
        let channel = Channel::new(handle);
        self.channels.insert(handle, channel);
        Ok(handle)
    }

    pub fn get_channel_address(&self, handle: ChannelHandle) -> Result<*mut u8, StormError> {
        match self.channels.get(&handle) {
            Some(channel) => Ok(channel.map_pointer),
            None => Err(StormError::NotFound),
        }
    }

    pub fn on_channel_messaged(&mut self, handle: ChannelHandle, handler: Option<Box<dyn Fn(ChannelHandle, u64)>>) -> Result<(), StormError> {
        match self.channels.get_mut(&handle) {
            Some(channel) => {
                channel.on_messaged = handler;
                Ok(())
            },
            None => Err(StormError::NotFound)
        }
    }

    pub fn emit_debug(&self, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Debug,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_information(&self, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Information,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_warning(&self, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Warning,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_error(&self, error: StormError, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(syscalls::EmitType::Error, error, Some(information_text))
    }

    pub fn wait_for_channel_message(&self, handle: ChannelHandle, message_id: u64, timeout_milliseconds: i32) -> Result<StormEvent, StormError> {
        syscalls::event_wait(Some(handle.raw_handle()), Some(StormAction::ChannelMessaged), Some(message_id), timeout_milliseconds)
    }

    // pub fn event_wait(&self) -> Result<Event, Error> {
    //     match syscalls::event_wait(handle, action, message, timeout_milliseconds)
    // }

    pub fn run(&self) -> Result<(), StormError> {
        // this is the main event loop of an application
        loop {
            let event = syscalls::event_wait(None, None, None, -1)?;

            match event {
                StormEvent::ServiceConnected(service_handle, channel_handle) => {
                    if let Some(service) = self.services.get(&service_handle) {
                        if let Some(handler) = &service.on_connected {
                            handler(service_handle, channel_handle);
                        }
                    }
                },
                StormEvent::ChannelMessaged(channel_handle, message_id) => {
                    if let Some(channel) = self.channels.get(&channel_handle) {
                        if let Some(handler) = &channel.on_messaged {
                            handler(channel_handle, message_id);
                        }
                    }
                },
                StormEvent::ChannelDestroyed(channel_handle) => {
                    if let Some(channel) = self.channels.get(&channel_handle) {
                        if let Some(handler) = &channel.on_destroyed {
                            handler(channel_handle);
                        }
                    }
                }
            }
        }
    }

    pub fn end(&self) -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
