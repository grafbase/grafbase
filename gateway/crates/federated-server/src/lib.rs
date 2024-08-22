//! Implements the self-hosted Grafbase gateway. It can run in a hybrid mode,
//! where we contact the schema registry in the API to fetch the latest schema
//! and send tracing and metrics to either our own or a 3rd party collector.

#![deny(missing_docs)]

mod hot_reload;
pub use error::Error;
pub use server::GdnResponse;
pub use server::GraphFetchMethod;

mod error;
mod server;

/// The crate result type.
pub type Result<T> = std::result::Result<T, Error>;

pub use server::{serve, ServerConfig, ServerRuntime};
