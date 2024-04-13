use super::gateway::{self, EngineWatcher};
use crate::server::gateway::GatewayConfig;
use crate::OtelReload;
use std::sync::Arc;
use tokio::sync::{oneshot, watch};

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
    #[cfg(not(feature = "lambda"))]
    FromApi {
        /// The access token for accessing the the API.
        access_token: ascii::AsciiString,
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
    #[cfg_attr(feature = "lambda", allow(unused_variables))]
    pub(crate) fn into_gateway(
        self,
        config: GatewayConfig,
        otel_reload: Option<oneshot::Sender<OtelReload>>,
    ) -> crate::Result<EngineWatcher> {
        let (sender, gateway) = watch::channel(None);

        match self {
            #[cfg(not(feature = "lambda"))]
            GraphFetchMethod::FromApi {
                access_token,
                graph_name,
                branch,
            } => {
                tokio::spawn(async move {
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(
                        &graph_name,
                        branch.as_deref(),
                        access_token,
                        sender,
                        config,
                        otel_reload,
                    )?
                    .poll()
                    .await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromLocal { federated_schema } => {
                let gateway = gateway::generate(&federated_schema, None, config)?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(gateway)
    }
}
