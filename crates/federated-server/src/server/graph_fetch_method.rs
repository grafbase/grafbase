use super::gateway::{self, GatewayRuntime, GraphDefinition};
use engine::Engine;
use gateway_config::Config;
use graph_ref::GraphRef;
use runtime_local::HooksWasi;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::{self};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

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
        reload_signal: Option<Receiver<(String, Config)>>,
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
            GraphFetchMethod::FromSchema {
                federated_sdl,
                reload_signal,
            } => {
                let gateway = gateway::generate(
                    GraphDefinition::Sdl(federated_sdl),
                    config,
                    hot_reload_config_path.clone(),
                    hooks.clone(),
                )
                .await?;

                sender.send(Some(Arc::new(gateway)))?;

                if let Some(reload_signal) = reload_signal {
                    tokio::spawn(async move {
                        let mut stream = ReceiverStream::new(reload_signal);

                        while let Some((sdl, config)) = stream.next().await {
                            let gateway = gateway::generate(
                                GraphDefinition::Sdl(sdl),
                                &config,
                                hot_reload_config_path.clone(),
                                hooks.clone(),
                            )
                            .await?;
                            sender.send(Some(Arc::new(gateway)))?;
                        }

                        Ok::<_, crate::Error>(())
                    });
                }
            }
        }

        Ok(())
    }
}
