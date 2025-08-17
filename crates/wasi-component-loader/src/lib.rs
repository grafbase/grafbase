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
pub mod extension;
mod http_client;
pub mod resources;
mod state;

#[cfg(test)]
mod tests;

use state::{ExtensionState, InstanceState};

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
