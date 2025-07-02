use std::path::PathBuf;

use super::events::UpdateEvent;
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
        /// The location of the schema file
        schema_path: PathBuf,
    },
    FromSchemaReloadable {
        current_dir: PathBuf,
        sdl_receiver: mpsc::Receiver<String>,
    },
}

impl GraphFetchMethod {
    /// Starts a producer that sends graph updates to the provided channel.
    ///
    /// This can happen in two ways: if providing a graph SDL, we send a new graph immediately.
    /// Alternatively, if a graph ref and access token is provided, the function returns
    /// immediately, and runs a background process to fetch the graph definition from object storage
    pub(crate) async fn start_producer(self, sender: mpsc::Sender<UpdateEvent>) -> crate::Result<()> {
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
                    use super::graph_updater::ObjectStorageUpdater;

                    ObjectStorageUpdater::new(graph_ref, access_token, sender)?.poll().await;

                    Ok::<_, crate::Error>(())
                });

                Ok(())
            }
            GraphFetchMethod::FromSchema {
                federated_sdl,
                schema_path,
            } => {
                sender
                    .send(UpdateEvent::Graph(GraphDefinition::Sdl(
                        Some(schema_path.clone()),
                        federated_sdl,
                    )))
                    .await
                    .expect("channel must be up");

                tokio::spawn(async move {
                    use super::graph_updater::SchemaFileGraphUpdater;
                    SchemaFileGraphUpdater::new(schema_path, sender).await.poll().await;
                });

                Ok(())
            }
            GraphFetchMethod::FromSchemaReloadable {
                current_dir,
                mut sdl_receiver,
            } => {
                tokio::spawn(async move {
                    while let Some(sdl) = sdl_receiver.recv().await {
                        if sender
                            .send(UpdateEvent::Graph(GraphDefinition::Sdl(Some(current_dir.clone()), sdl)))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                Ok(())
            }
        }
    }
}
