use std::sync::Arc;

use super::bus::{GraphReceiver, RequestReceiver, ResponseSender};
use engine::ServerError;
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

    pub async fn handler(mut self) -> Result<(), crate::Error> {
        let streams: [RouterStream; 2] = [
            Box::pin(ReceiverStream::new(self.graph_bus).map(RouterMessage::Graph)),
            Box::pin(ReceiverStream::new(self.request_bus).map(RouterMessage::request)),
        ];

        let mut stream = streams.merge();

        while let Some(message) = stream.next().await {
            match (message, self.engine.as_ref()) {
                (RouterMessage::Graph(graph), _) => {
                    self.engine = Some(new_engine(graph));
                }
                (RouterMessage::Request(request, response_sender), Some(engine)) => {
                    tokio::spawn(run_request(request, response_sender, Arc::clone(engine)));
                }
                (RouterMessage::Request(_, response_sender), None) => {
                    response_sender
                        .send(Err(ServerError::new(
                            "there are no subgraphs registered currently",
                            None,
                        )))
                        .ok();
                }
            }
        }

        Ok(())
    }
}

fn new_engine(graph: FederatedGraph) -> Arc<Engine> {
    Arc::new(Engine::new(graph.into()))
}

async fn run_request(request: engine::Request, response_sender: ResponseSender, engine: Arc<Engine>) {
    response_sender.send(engine.execute_request(request).await).ok();
}

enum RouterMessage {
    Graph(FederatedGraph),
    Request(engine::Request, ResponseSender),
}

impl RouterMessage {
    fn request((request, sender): (engine::Request, ResponseSender)) -> Self {
        RouterMessage::Request(request, sender)
    }
}

type RouterStream = BoxStream<'static, RouterMessage>;
