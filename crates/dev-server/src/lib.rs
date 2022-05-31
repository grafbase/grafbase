#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

pub mod errors;
mod server;

pub use server::start;
