#[macro_use]
extern crate lazy_static;

mod client;
mod server;
mod types;
mod client_to_server_calls;

pub use types::*;
pub use client::BogusClient;
pub use server::BogusServer;
pub use server::BogusServerImplementation;

// these will be moved to own files
pub use client_to_server_calls::render_call::RenderTypeArguments;
pub use client_to_server_calls::render_call::RenderHandleIterator;