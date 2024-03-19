//! Implements the self-hosted Grafbase gateway. It can run in a hybrid mode,
//! where we contact the schema registry in the API to fetch the latest schema
//! and send tracing and metrics to either our own or a 3rd party collector.

#![deny(missing_docs)]

pub use crate::config::Config;
pub use error::Error;
pub use server::GraphFetchMethod;

use std::net::SocketAddr;
use tokio::runtime;

mod config;
mod error;
mod server;

const THREAD_NAME: &str = "grafbase-gateway";

/// The crate result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Starts the self-hosted Grafbase gateway. If started with a schema path, will
/// not connect our API for changes in the schema and if started without, we poll
/// the schema registry every ten second for changes.
pub fn start(listen_addr: Option<SocketAddr>, config: Config, graph: GraphFetchMethod) -> Result<()> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()
        .map_err(|e| Error::InternalError(e.to_string()))?;

    runtime.block_on(server::serve(listen_addr, config, graph))?;

    Ok(())
}
