use uuid::Uuid;

use crate::error::Error;
use crate::handle::Handle;
use crate::syscalls;

pub fn create(protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Handle, Error> {
    syscalls::service_create(protocol_name, vendor_name, device_name, device_id)
}

pub fn connect(protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>) -> Result<Handle, Error> {
    syscalls::service_connect(protocol_name, vendor_name, device_name, device_id)
}