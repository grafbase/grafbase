#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

mod server;

pub use server::start;
