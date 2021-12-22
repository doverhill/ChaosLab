#[macro_use]
extern crate lazy_static;

mod types;
mod client;
mod server;

pub use types::*;
pub use client::BogusClient;
pub use server::BogusServer;
pub use server::BogusServerImplementation;
pub use server::Component;

mod simple_sum_call;
mod get_files_call;