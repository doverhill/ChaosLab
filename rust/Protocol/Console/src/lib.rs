#[macro_use]
extern crate lazy_static;

mod types;
pub use types::*;

mod client_to_server_calls;

mod server_to_client_calls;

mod server;
pub use server::ConsoleServer;
pub use server::ConsoleServerImplementation;

mod client;
pub use client::ConsoleClient;
pub use client::ConsoleClientImplementation;

