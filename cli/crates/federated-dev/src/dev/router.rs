use std::sync::Arc;

use super::bus::{GraphReceiver, RequestReceiver, ResponseSender};
use engine_v2::Engine;
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use graphql_composition::FederatedGraph;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct Router {
    graph_bus: GraphReceiver,
    request_bus: RequestReceiver,
    engine: Option<Arc<Engine>>,
}

impl Router {
    pub fn new(graph_bus: GraphReceiver, request_bus: RequestReceiver) -> Self {
        Self {
            graph_bus,
            request_bus,
            engine: None,
        }
    }

    pub async fn handler(mut self) {
        log::trace!("starting the router handler");

        let streams: [RouterStream; 2] = [
            Box::pin(ReceiverStream::new(self.graph_bus).map(RouterMessage::Graph)),
            Box::pin(ReceiverStream::new(self.request_bus).map(RouterMessage::request)),
        ];

        let mut stream = streams.merge();

        while let Some(message) = stream.next().await {
            match (message, self.engine.as_ref()) {
                (RouterMessage::Graph(graph), _) => {
                    log::trace!("router got a new graph");

                    self.engine = graph.map(new_engine);
                }
                (RouterMessage::Request(request, response_sender), Some(engine)) => {
                    log::trace!("router got a new request with an existing engine");

                    tokio::spawn(run_request(request, response_sender, Arc::clone(engine)));
                }
                (RouterMessage::Request(_, response_sender), None) => {
                    log::trace!("router got a new request with a missingengine");

                    response_sender.send(Err(RouterError::NoSubgraphs)).ok();
                }
            }
        }
    }
}

fn new_engine(graph: FederatedGraph) -> Arc<Engine> {
    Arc::new(Engine::new(graph.into()))
}

async fn run_request(request: engine::Request, response_sender: ResponseSender, engine: Arc<Engine>) {
    response_sender.send(Ok(engine.execute(request).await)).ok();
}

enum RouterMessage {
    Graph(Option<FederatedGraph>),
    Request(engine::Request, ResponseSender),
}

impl RouterMessage {
    fn request((request, sender): (engine::Request, ResponseSender)) -> Self {
        RouterMessage::Request(request, sender)
    }
}

type RouterStream = BoxStream<'static, RouterMessage>;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("there are no subgraphs registered currently")]
    NoSubgraphs,
}

pub type RouterResult<T> = Result<T, RouterError>;
