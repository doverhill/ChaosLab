use crate::{channel::Channel, service::Service, syscalls, ChannelHandle, ServiceHandle, StormAction, StormError, StormEvent};

use std::collections::HashMap;
use uuid::Uuid;

#[allow(dead_code)]
pub struct StormProcess {
    // pub struct StormProcess<'a> {
    name: String,
    services: HashMap<ServiceHandle, Service>,
    channels: HashMap<ChannelHandle, Channel>,
    // service_observers: Vec<&'a SO>,
    // channel_observers: Vec<&'a CO>,
}

impl Drop for StormProcess {
    // impl<'a> Drop for StormProcess<'a> {
    fn drop(&mut self) {
        self.end();
    }
}

impl StormProcess {
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

    pub fn create_service(&mut self, protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<ServiceHandle, StormError> {
        let handle = syscalls::service_create(protocol_name, vendor_name, device_name, device_id)?;
        self.services.insert(handle, Service::new());
        Ok(handle)
    }

    pub fn destroy_service(&mut self, handle: ServiceHandle) -> Result<(), StormError> {
        self.services.remove(&handle);
        syscalls::service_destroy(handle)
    }

    // pub fn attach_service_observer(&mut self, handle: ServiceHandle, observer: &'a mut SO) {
    //     if let Some(service) = self.services.get_mut(&handle) {
    //         service.attach_observer(observer);
    //     }
    // }

    // pub fn detach_service_observer(&mut self, handle: ServiceHandle, observer: &'a mut SO) {
    //     if let Some(service) = self.services.get_mut(&handle) {
    //         service.detach_observer(observer);
    //     }
    // }

    // pub fn attach_channel_observer(&mut self, handle: ChannelHandle, observer: &'a mut CO) {
    //     if let Some(channel) = self.channels.get_mut(&handle) {
    //         channel.attach_observer(observer);
    //     }
    // }

    // pub fn detach_channel_observer(&mut self, handle: ChannelHandle, observer: &'a mut CO) {
    //     if let Some(channel) = self.channels.get_mut(&handle) {
    //         channel.detach_observer(observer);
    //     }
    // }

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

    pub fn connect_to_service(&mut self, protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>, initial_size: usize) -> Result<ChannelHandle, StormError> {
        let handle = syscalls::service_connect(protocol_name, vendor_name, device_name, device_id)?;
        self.initialize_channel(handle, initial_size);
        // let channel = Channel::new(handle, initial_size);
        // self.channels.insert(handle, channel);
        Ok(handle)
    }

    pub fn initialize_channel(&mut self, handle: ChannelHandle, initial_size: usize) {
        let channel = Channel::new(handle, initial_size);
        self.channels.insert(handle, channel);
    }

    pub fn get_channel_address(&self, handle: ChannelHandle, id: usize) -> Result<*mut u8, StormError> {
        match self.channels.get(&handle) {
            Some(channel) => {
                if id == 0 {
                    Ok(channel.map_pointer_0)
                } else if id == 1 {
                    Ok(channel.map_pointer_1)
                } else {
                    Err(StormError::NotFound)
                }
            }
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

    pub fn signal_channel(handle: ChannelHandle) -> Result<(), StormError> {
        syscalls::channel_signal(handle)
    }

    pub fn wait_for_channel_signal(&self, handle: ChannelHandle, timeout_milliseconds: i32) -> Result<StormEvent, StormError> {
        syscalls::event_wait(Some(handle.raw_handle()), Some(StormAction::ChannelSignalled), timeout_milliseconds)
    }

    pub fn wait_for_event() -> Result<StormEvent, StormError> {
        syscalls::event_wait(None, None, -1)
    }

    // pub fn process_event(&mut self, event: StormEvent) {
    //     match event {
    //         StormEvent::ServiceConnected(service_handle, channel_handle) => {
    //             let channel = Channel::new(channel_handle, initial_size);
    //             self.channels.insert(handle, channel);

    //         },
    //         StormEvent::ChannelMessaged(channel_handle, message_id) => {
    //             if let Some(channel) = self.channels.get_mut(&channel_handle) {
    //                 for handler in channel.observers.iter_mut() {
    //                     handler.handle_channel_messaged(channel_handle, message_id);
    //                 }
    //                 // if let Some(handler) = &channel.on_messaged {
    //                 //     Self::emit_debug("invoking handler for channel message");
    //                 //     handler(channel_handle, message_id);
    //                 // }
    //             }
    //         },
    //         StormEvent::ChannelDestroyed(channel_handle) => {
    //             if let Some(channel) = self.channels.get_mut(&channel_handle) {
    //                 for handler in channel.observers.iter_mut() {
    //                     handler.handle_channel_destroyed(channel_handle);
    //                 }
    //                 // if let Some(handler) = &channel.on_destroyed {
    //                 //     Self::emit_debug("invoking handler for channel destroy");
    //                 //     handler(channel_handle);
    //                 // }
    //             }
    //         }
    //     }
    // }

    // pub fn run(&mut self) -> Result<(), StormError> {
    //     // this is the main event loop of an application
    //     loop {
    //         let event = syscalls::event_wait(None, None, None, -1)?;
    //         self.handle_event(event);
    //     }
    // }

    pub fn end(&self) -> ! {
        syscalls::process_destroy().unwrap();
        syscalls::cleanup();
        std::process::exit(0);
    }
}
