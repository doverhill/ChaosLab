#[macro_use]
extern crate lazy_static;

mod types;
pub use types::*;

mod client_to_server_calls;

mod server;
pub use server::BogusAutoServer;
pub use server::BogusAutoServerImplementation;

mod client;
pub use client::BogusAutoClient;
pub use client::BogusAutoClientImplementation;

