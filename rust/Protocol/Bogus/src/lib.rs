#[macro_use]
extern crate lazy_static;

mod client;
mod server;

pub use client::BogusClient;
pub use server::BogusServer;

mod simple_sum_client_call;
