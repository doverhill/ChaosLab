#[macro_use]
extern crate lazy_static;

mod client;
mod server;
mod types;

pub use types::*;
pub use client::BogusClient;
pub use server::BogusServer;
pub use server::BogusServerImplementation;

mod simple_sum_call;
mod get_files_call;
mod render_call;
mod get_next_call;

pub use render_call::RenderTypeArguments;
pub use render_call::RenderHandleIterator;