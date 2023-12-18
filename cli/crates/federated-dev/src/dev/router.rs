use std::sync::Arc;

use crate::ConfigReceiver;

use super::bus::{GraphReceiver, RequestReceiver, ResponseSender};
use engine::RequestHeaders;
use engine_v2::{Engine, EngineRuntime};
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use tokio_stream::wrappers::{ReceiverStream, WatchStream};

pub(crate) struct Router {
    graph: GraphReceiver,
    request_bus: RequestReceiver,
    engine: Option<Arc<Engine>>,
    config: ConfigReceiver,
}

impl Router {
    pub fn new(graph: GraphReceiver, request_bus: RequestReceiver, config: ConfigReceiver) -> Self {
        Self {
            graph,
            request_bus,
            engine: None,
            config,
        }
    }

    pub async fn handler(mut self) {
        log::trace!("starting the router handler");

        let streams: [RouterStream; 3] = [
            Box::pin(ReceiverStream::new(self.request_bus).map(RouterMessage::request)),
            Box::pin(WatchStream::new(self.graph.clone()).map(|_| RouterMessage::GraphUpdated)),
            Box::pin(WatchStream::new(self.config.clone()).map(|_| RouterMessage::ConfigUpdated)),
        ];

        let mut stream = streams.merge();

        while let Some(message) = stream.next().await {
            match (message, self.engine.as_ref()) {
                (RouterMessage::GraphUpdated, _) => {
                    log::trace!("router received a graph update");

                    self.engine = new_engine(&self.graph, &self.config)
                }
                (RouterMessage::ConfigUpdated, _) => {
                    log::trace!("router received a config update");

                    self.engine = new_engine(&self.graph, &self.config)
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

fn new_engine(graph: &GraphReceiver, config: &ConfigReceiver) -> Option<Arc<Engine>> {
    let graph = graph.borrow().clone()?;

    let config = engine_config_builder::build_config(&config.borrow(), graph);

    Some(Arc::new(Engine::new(
        config.into_latest().into(),
        EngineRuntime {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        },
    )))
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
    GraphUpdated,
    ConfigUpdated,
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
