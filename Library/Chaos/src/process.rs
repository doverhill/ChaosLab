use crate::{syscalls, ChannelHandle, ServiceHandle, StormAction, StormError, StormEvent, channel::Channel, service::Service, ServiceObserver, ChannelObserver};

use std::collections::HashMap;
use uuid::Uuid;

#[derive(PartialEq)]
pub struct StormProcess<'a, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {
// pub struct StormProcess<'a> {
    name: String,
    services: HashMap<ServiceHandle, Service<'a, SO>>,
    channels: HashMap<ChannelHandle, Channel<'a, CO>>,
    // service_observers: Vec<&'a SO>,
    // channel_observers: Vec<&'a CO>,
}

impl<'a, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> Drop for StormProcess<'a, SO, CO> {
// impl<'a> Drop for StormProcess<'a> {
    fn drop(&mut self) {
        self.end();
    }
}

impl<'a, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> StormProcess<'a, SO, CO> {
// impl<'a> StormProcess<'a> {
    pub fn new(name: &str) -> Result<Self, StormError> {
        syscalls::process_set_info(name)?;

        Ok(Self {
            name: name.to_string(),
            services: HashMap::new(),
            channels: HashMap::new(),
            // service_observers: Vec::new(),
            // channel_observers: Vec::new(),
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

    pub fn attach_service_observer(&mut self, handle: ServiceHandle, observer: &'a SO) {
        if let Some(service) = self.services.get_mut(&handle) {
            service.attach_observer(observer);
        }
    }

    pub fn detach_service_observer(&mut self, handle: ServiceHandle, observer: &'a SO) {
        if let Some(service) = self.services.get_mut(&handle) {
            service.detach_observer(observer);
        }
    }

    pub fn attach_channel_observer(&mut self, handle: ChannelHandle, observer: &'a CO) {
        if let Some(channel) = self.channels.get_mut(&handle) {
            channel.attach_observer(observer);
        }
    }

    pub fn detach_channel_observer(&mut self, handle: ChannelHandle, observer: &'a CO) {
        if let Some(channel) = self.channels.get_mut(&handle) {
            channel.detach_observer(observer);
        }
    }

    // pub fn on_service_connected(
    //     &mut self,
    //     handle: ServiceHandle,
    //     handler: impl Fn(ServiceHandle, ChannelHandle) + 'a,
    // ) -> Result<(), StormError> {
    //     match self.services.get_mut(&handle) {
    //         Some(service) => {
    //             service.on_connected = Some(Box::new(handler));
    //             Ok(())
    //         },
    //         None => Err(StormError::NotFound)
    //     }
    // }

    // pub fn clear_on_service_connected(
    //     &mut self,
    //     handle: ServiceHandle,
    // ) -> Result<(), StormError> {
    //     match self.services.get_mut(&handle) {
    //         Some(service) => {
    //             service.on_connected = None;
    //             Ok(())
    //         },
    //         None => Err(StormError::NotFound)
    //     }
    // }

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

    // pub fn on_channel_messaged(&mut self, handle: ChannelHandle, handler: Option<Box<dyn Fn(ChannelHandle, u64)>>) -> Result<(), StormError> {
    //     match self.channels.get_mut(&handle) {
    //         Some(channel) => {
    //             channel.on_messaged = handler;
    //             Ok(())
    //         },
    //         None => Err(StormError::NotFound)
    //     }
    // }

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
                Self::emit_debug("invoking handler for service connect");
                if let Some(service) = self.services.get(&service_handle) {
                    Self::emit_debug("found service");
                    for handler in service.observers.iter() {
                        handler.handle_service_connected(service_handle, channel_handle);
                    }
                    // if let Some(handler) = &service.on_connected {
                    //     Self::emit_debug("calling handler for service connect");
                    //     handler(service_handle, channel_handle);
                    // }
                }
            },
            StormEvent::ChannelMessaged(channel_handle, message_id) => {
                Self::emit_debug("invoking handler for channel message");
                if let Some(channel) = self.channels.get(&channel_handle) {
                    Self::emit_debug("found service");
                    for handler in channel.observers.iter() {
                        handler.handle_channel_messaged(channel_handle, message_id);
                    }
                    // if let Some(handler) = &channel.on_messaged {
                    //     Self::emit_debug("invoking handler for channel message");
                    //     handler(channel_handle, message_id);
                    // }
                }
            },
            StormEvent::ChannelDestroyed(channel_handle) => {
                Self::emit_debug("invoking handler for channel destroy");
                if let Some(channel) = self.channels.get(&channel_handle) {
                    Self::emit_debug("found service");
                    for handler in channel.observers.iter() {
                        handler.handle_channel_destroyed(channel_handle);
                    }
                    // if let Some(handler) = &channel.on_destroyed {
                    //     Self::emit_debug("invoking handler for channel destroy");
                    //     handler(channel_handle);
                    // }
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
