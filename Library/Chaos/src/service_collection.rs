use crate::{ StormHandle, syscalls, StormError };
use uuid::Uuid;

pub struct ServiceCollection {
    service_handles: Vec<StormHandle>
}

impl ServiceCollection {
    pub fn new() -> Self {
        ServiceCollection { service_handles: vec![] }
    }

    pub fn create(&mut self, protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<StormHandle, StormError> {
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

    pub fn destroy(&mut self, service_handle: StormHandle) {
        match syscalls::service_destroy(service_handle)
    }
}