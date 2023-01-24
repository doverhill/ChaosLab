use crate::{ Handle, syscalls, Error };
use uuid::Uuid;

pub struct ServiceCollection {
    service_handles: Vec<Handle>
}

impl ServiceCollection {
    pub fn new() -> Self {
        ServiceCollection { service_handles: vec![] }
    }

    pub fn create(&mut self, protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Handle, Error> {
        match syscalls::service_create(protocol_name, vendor_name, device_name, device_id) {
            Ok(handle) => {
                self.service_handles.push(handle);
                Ok(handle)
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}