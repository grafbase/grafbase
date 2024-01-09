use std::sync::Arc;

use crate::ConfigReceiver;

use super::bus::{GraphReceiver, RequestReceiver, ResponseSender};
use engine::RequestHeaders;
use engine_v2::EngineRuntime;
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use gateway_v2::Gateway;
use tokio_stream::wrappers::{ReceiverStream, WatchStream};

pub(crate) struct Router {
    graph: GraphReceiver,
    request_bus: RequestReceiver,
    gateway: Option<Arc<Gateway>>,
    config: ConfigReceiver,
}

impl Router {
    pub fn new(graph: GraphReceiver, request_bus: RequestReceiver, config: ConfigReceiver) -> Self {
        Self {
            graph,
            request_bus,
            gateway: None,
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
            match (message, self.gateway.as_ref()) {
                (RouterMessage::GraphUpdated, _) => {
                    log::trace!("router received a graph update");

                    self.gateway = new_engine(&self.graph, &self.config).await
                }
                (RouterMessage::ConfigUpdated, _) => {
                    log::trace!("router received a config update");

                    self.gateway = new_engine(&self.graph, &self.config).await
                }
                (RouterMessage::Request(request, headers, response_sender), Some(gateway)) => {
                    log::trace!("router got a new request with an existing engine");

                    tokio::spawn(run_request(request, headers, response_sender, Arc::clone(gateway)));
                }
                (RouterMessage::Request(_, _, response_sender), None) => {
                    log::trace!("router got a new request with a missing engine");

                    response_sender.send(Err(RouterError::NoSubgraphs)).ok();
                }
            }
        }
    }
}

async fn new_engine(graph: &GraphReceiver, config: &ConfigReceiver) -> Option<Arc<Gateway>> {
    let graph = graph.borrow().clone()?;

    let config = engine_config_builder::build_config(&config.borrow(), graph);
    Some(Arc::new(Gateway::new(
        config.into_latest().into(),
        EngineRuntime {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        },
        runtime_local::InMemoryKvStore::runtime_kv(),
    )))
}

async fn run_request(
    request: engine::Request,
    headers: RequestHeaders,
    response_sender: ResponseSender,
    engine: Arc<Gateway>,
) {
    response_sender
        .send(
            engine
                .execute(request, headers, serde_json::to_vec)
                .await
                .map_err(Into::into),
        )
        .ok();
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
    #[error("Serialization failure: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type RouterResult<T> = Result<T, RouterError>;
