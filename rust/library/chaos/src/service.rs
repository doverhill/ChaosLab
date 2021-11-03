use crate::{ syscalls, channel::Channel, error::Error, handle::Handle, process::Process };
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
    on_connect: Option<fn(&Arc<Mutex<Service>>, Arc<Mutex<Channel>>) -> ()>
}

impl Service {
    fn new(handle: Handle) -> Service {
        Service {
            handle: handle,
            on_connect: None
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
        let result = syscalls::service_connect(protocol_name, vendor_name, device_name, device_id);
    
        match result {
            Ok(handle) => {
                Ok(Channel::new(handle, size))
            },
            Err(error) => {
                Err(error)
            }
        }
    }
    
    pub fn on_connect(&mut self, handler: fn(&Arc<Mutex<Service>>, Arc<Mutex<Channel>>) -> ()) -> Option<Error> {
        match self.on_connect {
            Some(_) => {
                Some(Error::AlreadyExists)
            },
            None => {
                self.on_connect = Some(handler);
                None
            }
        }
    }

    pub(crate) fn connected(handle: Handle, channel_handle: Handle) {
        Process::emit_debug(&format!("Service connect on {} -> channel {}", handle, channel_handle));

        let services = SERVICES.lock().unwrap();
        if let Some(service_wrap) = services.get(&handle) {
            let service = service_wrap.lock().unwrap();
            if let Some(handler) = service.on_connect {
                let channel = Channel::new(channel_handle, 4096);
                drop(service); // release mutex
                handler(service_wrap, channel);
            }
        }
    }

    pub fn destroy(self) -> Option<Error> {
        syscalls::service_destroy(self.handle)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        syscalls::service_destroy(self.handle);
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[SERVICE: handle={}]", self.handle)
    }
}
