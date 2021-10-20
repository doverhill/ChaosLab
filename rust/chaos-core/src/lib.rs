#[macro_use]
extern crate lazy_static;

pub mod channel;
pub mod handle;
pub mod action;
pub mod error;
pub mod syscalls;
pub mod process;
pub mod service;

pub fn done() -> () {
    syscalls::cleanup();
}