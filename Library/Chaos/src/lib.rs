#![feature(thread_id_value)]
#![feature(local_key_cell_methods)]

mod event;
mod syscalls;
mod channel;
mod service;
mod handle;
mod error;
mod process;
// mod service_collection;
// mod channel_collection;

pub use event::{StormAction, StormEvent};
pub use process::StormProcess;
pub use error::StormError;
pub use handle::{ServiceHandle, ChannelHandle};
// pub use service_collection::ServiceCollection;
// pub use channel_collection::ChannelCollection;
