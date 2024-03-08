//! Implements the self-hosted Grafbase gateway. It can run in a hybrid mode,
//! where we contact the schema registry in the API to fetch the latest schema
//! and send tracing and metrics to either our own or a 3rd party collector.

#![deny(missing_docs)]

use std::{fs, net::SocketAddr, path::Path};
use tokio::runtime;

pub use error::Error;
pub use server::GraphFetchMethod;

mod config;
mod error;
mod server;

const THREAD_NAME: &str = "grafbase-gateway";

/// The crate result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Starts the self-hosted Grafbase gateway. If started with a schema path, will
/// not connect our API for changes in the schema and if started without, we poll
/// the schema registry every ten second for changes.
pub fn start(listen_addr: Option<SocketAddr>, config_path: &Path, graph: GraphFetchMethod) -> crate::Result<()> {
    let config = fs::read_to_string(config_path).map_err(Error::ConfigNotFound)?;
    let config = toml::from_str(&config).map_err(Error::TomlValidation)?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()
        .map_err(|e| Error::InternalError(e.to_string()))?;

    runtime.block_on(server::serve(listen_addr, config, graph))?;

    Ok(())
}
