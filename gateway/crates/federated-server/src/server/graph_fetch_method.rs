use super::{
    gateway::{self, GatewayWatcher},
    graph_updater::GraphUpdater,
};
use crate::server::gateway::GatewayConfig;
use ascii::AsciiString;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::Level;

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
    FromApi {
        /// The access token for accessing the the API.
        access_token: AsciiString,
        /// The name of the graph
        graph_name: String,
        /// The graph branch
        branch: Option<String>,
    },
    /// The schema is loaded from disk. No access to the Grafbase API.
    FromLocal {
        /// Static federated graph from a file
        federated_schema: String,
    },
}

impl GraphFetchMethod {
    /// Converts the fetch method into an eventually existing gateway. This can happen
    /// in two ways: if providing a graph SDL, we a new gateway immediately. Alternatively,
    /// if a graph ref and access token is provided, the function returns immediately, and
    /// the gateway will be available eventually when the GDN responds with a working graph.
    pub(crate) fn into_gateway(self, config: GatewayConfig) -> crate::Result<GatewayWatcher> {
        let (sender, gateway) = watch::channel(None);

        match self {
            GraphFetchMethod::FromApi {
                access_token,
                graph_name,
                branch,
            } => {
                tokio::spawn(async move {
                    GraphUpdater::new(&graph_name, branch.as_deref(), access_token, sender, config)?
                        .poll()
                        .await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromLocal { federated_schema } => {
                tracing::event!(
                    Level::INFO,
                    message = "creating a new gateway",
                    operation_limits = config.operation_limits.is_some(),
                    introspection_enabled = config.enable_introspection,
                    authentication = config.authentication.is_some(),
                );

                let gateway = gateway::generate(&federated_schema, None, config)?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(gateway)
    }
}
