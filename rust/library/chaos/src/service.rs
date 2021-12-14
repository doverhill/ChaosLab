use crate::{ syscalls, Channel, Error, Handle, Process };
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref SERVICES: Mutex<HashMap<Handle, Arc<Mutex<Service>>>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct Service {
    handle: Handle,
    connect_handler: Option<fn(&Arc<Mutex<Service>>, Arc<Mutex<Channel>>) -> ()>
}

impl Service {
    fn new(handle: Handle) -> Service {
        Service {
            handle: handle,
            connect_handler: None
        }
    }

    pub fn create(protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Arc<Mutex<Service>>, Error> {
        match syscalls::service_create(protocol_name, vendor_name, device_name, device_id) {
            Ok(handle) => {
                let service = Arc::new(Mutex::new(Service::new(handle)));
                let services = &mut *SERVICES.lock().unwrap();
                services.insert(handle, service.clone());
                Ok(service)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    pub fn connect(protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>, size: usize) -> Result<Arc<Mutex<Channel>>, Error> {
        match syscalls::service_connect(protocol_name, vendor_name, device_name, device_id) {
            Ok(handle) => {
                Ok(Channel::new(handle, size))
            },
            Err(error) => {
                Err(error)
            }
        }
    }
    
    pub fn on_connect(&mut self, handler: fn(&Arc<Mutex<Service>>, Arc<Mutex<Channel>>) -> ()) -> Result<(), Error> {
        match self.connect_handler {
            Some(_) => {
                Err(Error::AlreadyExists)
            },
            None => {
                self.connect_handler = Some(handler);
                Ok(())
            }
        }
    }

    pub(crate) fn connected(handle: Handle, channel_handle: Handle) {
        Process::emit_debug(&format!("Service connect on {} -> channel {}", handle, channel_handle)).unwrap();

        let services = SERVICES.lock().unwrap();
        if let Some(service_wrap) = services.get(&handle) {
            let service = service_wrap.lock().unwrap();
            if let Some(handler) = service.connect_handler {
                let channel = Channel::new(channel_handle, 4096);
                drop(service); // release mutex
                handler(service_wrap, channel);
            }
        }
    }

    pub fn destroy(self) -> Result<(), Error> {
        syscalls::service_destroy(self.handle)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        syscalls::service_destroy(self.handle).unwrap();
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[SERVICE: handle={}]", self.handle)
    }
}
