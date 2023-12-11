use std::sync::Arc;

use super::bus::{GraphReceiver, RequestReceiver, ResponseSender};
use engine::RequestHeaders;
use engine_v2::{Engine, EngineRuntime};
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct Router {
    graph_bus: GraphReceiver,
    request_bus: RequestReceiver,
    engine: Option<Arc<Engine>>,
    config: FederatedGraphConfig,
}

impl Router {
    pub fn new(graph_bus: GraphReceiver, request_bus: RequestReceiver, config: FederatedGraphConfig) -> Self {
        Self {
            graph_bus,
            request_bus,
            engine: None,
            config,
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

                    self.engine = graph.map(|graph| new_engine(&self.config, graph));
                }
                (RouterMessage::Request(request, headers, response_sender), Some(engine)) => {
                    log::trace!("router got a new request with an existing engine");

                    tokio::spawn(run_request(request, headers, response_sender, Arc::clone(engine)));
                }
                (RouterMessage::Request(_, _, response_sender), None) => {
                    log::trace!("router got a new request with a missing engine");

                    response_sender.send(Err(RouterError::NoSubgraphs)).ok();
                }
            }
        }
    }
}

fn new_engine(config: &FederatedGraphConfig, graph: FederatedGraph) -> Arc<Engine> {
    let config = engine_config_builder::build_config(config, graph);

    Arc::new(Engine::new(
        config.into_latest().into(),
        EngineRuntime {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        },
    ))
}

async fn run_request(
    request: engine::Request,
    headers: RequestHeaders,
    response_sender: ResponseSender,
    engine: Arc<Engine>,
) {
    response_sender.send(Ok(engine.execute(request, headers).await)).ok();
}

enum RouterMessage {
    Graph(Option<FederatedGraph>),
    Request(engine::Request, RequestHeaders, ResponseSender),
}

impl RouterMessage {
    fn request((request, headers, sender): (engine::Request, RequestHeaders, ResponseSender)) -> Self {
        RouterMessage::Request(request, headers, sender)
    }
}

type RouterStream = BoxStream<'static, RouterMessage>;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("there are no subgraphs registered currently")]
    NoSubgraphs,
}

pub type RouterResult<T> = Result<T, RouterError>;
