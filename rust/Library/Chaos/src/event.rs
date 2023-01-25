use crate::{ Handle, Action };

pub struct Event {
    handle: Handle,
    argument_handle: Handle,
    action: Action,
    parameter: u64
}