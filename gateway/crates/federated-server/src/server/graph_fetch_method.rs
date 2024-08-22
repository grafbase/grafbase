use super::gateway::{self, GatewayRuntime};
use engine_v2::Engine;
use gateway_config::Config;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::watch;

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
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
    pub(crate) async fn start(
        self,
        config: &Config,
        hot_reload_config_path: Option<PathBuf>,
        sender: watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>,
    ) -> crate::Result<()> {
        #[cfg(feature = "lambda")]
        if matches!(self, GraphFetchMethod::FromApi { .. }) {
            return Err(crate::Error::InternalError(
                "Cannot fetch schema with graph in lambda mode.".to_string(),
            ));
        }

        match self {
            GraphFetchMethod::FromApi {
                access_token,
                graph_name,
                branch,
            } => {
                let config = config.clone();
                tokio::spawn(async move {
                    let config = config.clone();
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(&graph_name, branch.as_deref(), access_token, sender, config)?
                        .poll()
                        .await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromLocal { federated_schema } => {
                let gateway = gateway::generate(&federated_schema, None, config, hot_reload_config_path).await?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(())
    }
}
