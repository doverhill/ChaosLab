use crate::{ StormHandle, StormAction };

#[derive(Debug)]
pub struct StormEvent {
    handle: StormHandle,
    argument_handle: StormHandle,
    action: StormAction,
    parameter: u64
}