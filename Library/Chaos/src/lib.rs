#![feature(thread_id_value)]
#![feature(local_key_cell_methods)]

mod event;
mod syscalls;
mod channel;
mod service;
mod handle;
mod error;
mod process;
mod client_store;

pub use event::{StormAction, StormEvent};
pub use process::StormProcess;
pub use error::StormError;
pub use handle::{ServiceHandle, ChannelHandle};
pub use client_store::ClientStore;