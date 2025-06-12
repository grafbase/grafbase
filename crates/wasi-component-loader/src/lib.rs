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
mod error;
pub mod extension;
mod http_client;
pub mod resources;
mod state;

#[cfg(test)]
mod tests;

use tonic13 as tonic;

pub use context::{ContextMap, SharedContext};
pub use crossbeam::channel::Sender;
pub use crossbeam::sync::WaitGroup;
pub use error::{Error, ErrorResponse};
pub use extension::api::wit::Error as GuestError;

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;
/// The guest result type
pub type GuestResult<T> = std::result::Result<T, GuestError>;
/// The gateway result type
pub type GatewayResult<T> = std::result::Result<T, ErrorResponse>;

use state::WasiState;
