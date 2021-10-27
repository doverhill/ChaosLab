use crate::{ syscalls, process, channel::Channel, error::Error, handle::Handle };
use uuid::Uuid;

pub fn create(protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Handle, Error> {
    syscalls::service_create(protocol_name, vendor_name, device_name, device_id)
}

pub fn connect(protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>) -> Result<Handle, Error> {
    let result = syscalls::service_connect(protocol_name, vendor_name, device_name, device_id);

    match result {
        Ok(handle) => {
            let channel = Channel::new(handle);
            let handle = channel.channel_handle;
            process::register_channel(channel);
            Ok(handle)
        },
        Err(error) => {
            Err(error)
        }
    }
}
