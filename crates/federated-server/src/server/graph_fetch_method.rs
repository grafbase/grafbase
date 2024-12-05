use super::engine_reloader::GraphSender;
use super::gateway::GraphDefinition;
use graph_ref::GraphRef;
use tokio::sync::mpsc;

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
    FromSchemaReloadable {
        sdl_receiver: mpsc::Receiver<String>,
    },
}

impl GraphFetchMethod {
    /// Converts the fetch method into an eventually existing graph definition. This can happen
    /// in two ways: if providing a graph SDL, we send a new graph immediately. Alternatively,
    /// if a graph ref and access token is provided, the function returns immediately, and
    /// runs a background process to fetch the graph definition from the GDN
    pub(crate) async fn start(self, sender: GraphSender) -> crate::Result<()> {
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
                tokio::spawn(async move {
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(graph_ref, access_token, sender)?.poll().await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromSchema { federated_sdl } => {
                sender.send(GraphDefinition::Sdl(federated_sdl)).await?;
            }
            GraphFetchMethod::FromSchemaReloadable { mut sdl_receiver } => {
                tokio::spawn(async move {
                    while let Some(sdl) = sdl_receiver.recv().await {
                        if sender.send(GraphDefinition::Sdl(sdl)).await.is_err() {
                            break;
                        }
                    }
                });
            }
        }

        Ok(())
    }
}
