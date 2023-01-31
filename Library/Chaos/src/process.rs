use crate::{syscalls, ChannelHandle, ServiceHandle, StormAction, StormError, StormEvent, channel::Channel, service::Service};

use std::collections::HashMap;
use uuid::Uuid;

pub struct StormProcess<'a> {
    name: String,
    services: HashMap<ServiceHandle, Service<'a>>,
    channels: HashMap<ChannelHandle, Channel<'a>>,
}

impl<'a> Drop for StormProcess<'a> {
    fn drop(&mut self) {
        self.end();
    }
}

impl<'a> StormProcess<'a> {
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
        handler: impl Fn(ServiceHandle, ChannelHandle) + 'a,
    ) -> Result<(), StormError> {
        match self.services.get_mut(&handle) {
            Some(service) => {
                service.on_connected = Some(Box::new(handler));
                Ok(())
            },
            None => Err(StormError::NotFound)
        }
    }

    pub fn clear_on_service_connected(
        &mut self,
        handle: ServiceHandle,
    ) -> Result<(), StormError> {
        match self.services.get_mut(&handle) {
            Some(service) => {
                service.on_connected = None;
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

    pub fn emit_debug(information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Debug,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_information(information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Information,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_warning(information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(
            syscalls::EmitType::Warning,
            StormError::None,
            Some(information_text),
        )
    }

    pub fn emit_error(error: StormError, information_text: &str) -> Result<(), StormError> {
        syscalls::process_emit(syscalls::EmitType::Error, error, Some(information_text))
    }

    pub fn send_channel_message(handle: ChannelHandle, message_id: u64) -> Result<(), StormError> {
        syscalls::channel_message(handle, message_id)
    }

    pub fn wait_for_channel_message(&self, handle: ChannelHandle, message_id: u64, timeout_milliseconds: i32) -> Result<StormEvent, StormError> {
        syscalls::event_wait(Some(handle.raw_handle()), Some(StormAction::ChannelMessaged), Some(message_id), timeout_milliseconds)
    }

    pub fn wait_for_event() -> Result<StormEvent, StormError> {
        syscalls::event_wait(None, None, None, -1)
    }

    pub fn handle_event(&self, event: StormEvent) {
        Self::emit_debug("handling event");

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

    pub fn run(&self) -> Result<(), StormError> {
        // this is the main event loop of an application
        loop {
            let event = syscalls::event_wait(None, None, None, -1)?;
            self.handle_event(event);
        }
    }

    pub fn end(&self) -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
