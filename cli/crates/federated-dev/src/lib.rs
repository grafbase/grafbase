//! # Federated development mode for Grafbase.
//!
//! The system listens in the given port, and exposes an `/admin` endpoint with one mutation:
//!
//!   - `publishSubgraph`: with input parameters `name` and `url`
//!
//! When called, it will introspect the given url, and if the introspection returns a valid
//! GraphQL schema, it will be composed with the existing subgraphs into a federated graph.
//!
//! Every second, the system refreshes the stored subgraphs, and if any of them disappeared
//! (the dev server is down) or changed, the changes are reflected into a new federated graph.
//!
//! Whenever the federated graph changes, the router gets notified, which should trigger a restart.
//!
//! Calls to the `/graphql` endpoint should be GraphQL calls, sent to the router, which then
//! handles the request.
//!
//! ## Actors
//!
//! The system consists of five actors:
//!
//! - `Admin` is a Axum HTTP server, which modifies the federated graph in runtime.
//! - `Composer` manages stored subgraphs, composes them and communicates with the router, refresher and admin
//! - `Refresher` gets a list of urls, queries them and decides if the returned subgraph triggers a recompose
//! - `Router` runs the router, which answers to the user's GraphQL queries and gets the federated graph from the composer
//! - `Ticker` sends a tick every second to the composer, which then calls refresher to refresh the stored graphs
//!
//! ## Workflow
//!
//! When executing `gb publish --dev --name foo --url http://foo.bar/lol`:
//!
//! - A GraphQL query hits the admin endpoint
//! - The admin calls composer to introspect the schema through a channel
//! - When the schema is introspected, it calls the composer with a composing message through a channel
//! - If the composition was successful, it returns a success. Otherwise an error.
//!
//! A ticker ticks after one second:
//!
//! - The ticker sends a message to the composer to initialize a refresh
//! - The composer sends the names, urls and hashes of the graphs to the refresher
//! - The refresher iterates over the vector, introspects, if the result is different than the previous
//!   one, sends a message to the composer to recompose with the new subgraph. If it's the same graph,
//!   it does nothing and continues iteration.
//! - The iteration will go through all the graphs, and composes every time there is a change.
//! - If the introspection fails to respond or fails to compose, the subgraph is removed and a new
//!   federated graph to the router.

#![deny(missing_docs)]

mod dev;
mod error;
mod events;
mod subgraph;

use std::net::SocketAddr;

pub use self::{
    error::Error,
    events::{subscribe, FederatedDevEvent},
};

use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use tokio::runtime::Builder;
use url::Url;

/// FederatedGraphConfig should be provided to federated-dev via this watch::Receiver type
pub type ConfigWatcher = tokio::sync::watch::Receiver<FederatedGraphConfig>;

/// Adds a subgraph to the running dev system.
pub fn add_subgraph(name: &str, url: &Url, dev_api_port: u16, headers: Vec<(&str, &str)>) -> Result<(), Error> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| Error::internal(error.to_string()))?;

    runtime.block_on(subgraph::add(name, url, dev_api_port, headers))
}

/// Runs the federated dev system.
pub async fn run(
    listen_address: SocketAddr,
    config: ConfigWatcher,
    graph: Option<FederatedGraph>,
) -> Result<(), Error> {
    dev::run(listen_address, config, graph).await
}
