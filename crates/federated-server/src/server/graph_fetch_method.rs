use super::gateway::{self, GatewayRuntime, GraphDefinition};
use engine_v2::Engine;
use gateway_config::Config;
use graph_ref::GraphRef;
use runtime_local::HooksWasi;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::watch;

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
    FromGraphRef {
        /// The access token for accessing the the API.
        access_token: ascii::AsciiString,
        graph_ref: GraphRef,
    },
    /// The schema is loaded from disk. No access to the Grafbase API.
    FromSchema {
        /// Static federated graph from a file
        federated_sdl: String,
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
        hooks: HooksWasi,
    ) -> crate::Result<()> {
        #[cfg(feature = "lambda")]
        if matches!(self, GraphFetchMethod::FromGraphRef { .. }) {
            return Err(crate::Error::InternalError(
                "Cannot fetch schema with graph in lambda mode.".to_string(),
            ));
        }

        match self {
            GraphFetchMethod::FromGraphRef {
                access_token,
                graph_ref,
            } => {
                let config = config.clone();
                tokio::spawn(async move {
                    let config = config.clone();
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(graph_ref, access_token, sender, config, hooks)?
                        .poll()
                        .await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromSchema { federated_sdl } => {
                let gateway = gateway::generate(
                    GraphDefinition::Sdl(federated_sdl),
                    config,
                    hot_reload_config_path,
                    hooks,
                )
                .await?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(())
    }
}
