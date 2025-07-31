mod object_storage;
mod schema_file;

use std::path::PathBuf;

use graph_ref::GraphRef;
use tokio::sync::mpsc;

pub use object_storage::*;
pub use schema_file::*;

use crate::{AccessToken, events::UpdateEvent, graph::Graph};

/// The method of running the gateway.
pub enum GraphLoader {
    /// The schema is fetched in regular intervals from the Grafbase API.
    FromGraphRef {
        /// The access token for accessing the the API.
        access_token: AccessToken,
        graph_ref: GraphRef,
    },
    /// The schema is loaded from disk. No access to the Grafbase API.
    FromSchemaFile {
        /// The location of the schema file
        path: PathBuf,
    },
    FromChannel {
        sdl_receiver: mpsc::Receiver<String>,
    },
}

impl GraphLoader {
    /// Starts a producer that sends graph updates to the provided channel.
    ///
    /// This can happen in two ways: if providing a graph SDL, we send a new graph immediately.
    /// Alternatively, if a graph ref and access token is provided, the function returns
    /// immediately, and runs a background process to fetch the graph definition from object storage
    pub(crate) async fn start_producer(self, sender: mpsc::Sender<UpdateEvent>) -> crate::Result<()> {
        #[cfg(feature = "lambda")]
        if matches!(self, GraphLoader::FromGraphRef { .. }) {
            return Err(crate::Error::InternalError(
                "Cannot fetch schema with graph in lambda mode.".to_string(),
            ));
        }

        match self {
            GraphLoader::FromGraphRef {
                access_token,
                graph_ref,
            } => {
                tokio::spawn(async move {
                    ObjectStorageUpdater::new(graph_ref, access_token, sender)?.poll().await;

                    Ok::<_, crate::Error>(())
                });

                Ok(())
            }
            GraphLoader::FromSchemaFile { path } => {
                let sdl = std::fs::read_to_string(&path).map_err(|err| {
                    crate::Error::InternalError(format!("could not read federated schema file: {err}"))
                })?;

                sender
                    .send(UpdateEvent::Graph(Graph::FromText { sdl }))
                    .await
                    .expect("channel must be up");

                tokio::spawn(async move {
                    SchemaFileGraphUpdater::new(path, sender).await.poll().await;
                });

                Ok(())
            }
            GraphLoader::FromChannel { mut sdl_receiver } => {
                tokio::spawn(async move {
                    while let Some(sdl) = sdl_receiver.recv().await {
                        if sender.send(UpdateEvent::Graph(Graph::FromText { sdl })).await.is_err() {
                            break;
                        }
                    }
                });

                Ok(())
            }
        }
    }
}
