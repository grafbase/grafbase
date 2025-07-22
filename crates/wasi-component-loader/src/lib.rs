//! # Customer hooks with WebAssembly component model
//!
//! This crate provides library support to load and run custom code compiled as a [WebAssembly component].
//! The calling code in this crate is called "host" and the called code "guest".
//!
//! It is important the compiled WebAssembly code implements at least the minimal required types and interfaces.
//! More on those on the crate README.

mod cache;
mod cbor;
mod config;
mod context;
pub mod extension;
mod http_client;
pub mod resources;
mod state;

#[cfg(test)]
mod tests;

use tonic13 as tonic;

pub use context::WasmContext;
pub use crossbeam::channel::Sender;
pub use crossbeam::sync::WaitGroup;
pub use extension::api::wit::Error as GuestError;

use state::{ExtensionState, InstanceState};
