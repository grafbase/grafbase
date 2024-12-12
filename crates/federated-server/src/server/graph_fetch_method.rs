use super::gateway::GraphDefinition;
use futures_lite::{stream, StreamExt};
use graph_ref::GraphRef;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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

pub type GraphStream = stream::Boxed<GraphDefinition>;

impl GraphFetchMethod {
    /// Converts the fetch method into a stream of graph definitions.
    ///
    /// This can happen in two ways: if providing a graph SDL, we return a new graph immediately.
    /// Alternatively, if a graph ref and access token is provided, the function returns
    /// immediately, and runs a background process to fetch the graph definition from the GDN
    pub(crate) async fn into_stream(self) -> crate::Result<GraphStream> {
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
                let (sender, receiver) = mpsc::channel(4);

                tokio::spawn(async move {
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(graph_ref, access_token, sender)?.poll().await;

                    Ok::<_, crate::Error>(())
                });

                Ok(ReceiverStream::new(receiver).boxed())
            }
            GraphFetchMethod::FromSchema { federated_sdl } => {
                Ok(once_then_pending_stream(GraphDefinition::Sdl(federated_sdl)).boxed())
            }
            GraphFetchMethod::FromSchemaReloadable { mut sdl_receiver } => {
                let (sender, receiver) = mpsc::channel(4);

                tokio::spawn(async move {
                    while let Some(sdl) = sdl_receiver.recv().await {
                        if sender.send(GraphDefinition::Sdl(sdl)).await.is_err() {
                            break;
                        }
                    }
                });

                Ok(ReceiverStream::new(receiver).boxed())
            }
        }
    }
}

/// Returns a stream that returns item on first poll and then remains pending
/// forever.
fn once_then_pending_stream<T>(item: T) -> impl futures_lite::Stream<Item = T> {
    stream::once(item).chain(stream::pending())
}
