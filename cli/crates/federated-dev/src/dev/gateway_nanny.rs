use std::sync::Arc;

use crate::ConfigWatcher;

use super::bus::{GatewaySender, GraphWatcher};
use engine_v2::{Engine, EngineEnv};
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use tokio_stream::wrappers::WatchStream;

/// The GatewayNanny looks after the `Gateway` - on updates to the graph or config it'll
/// create a new `Gateway` and publish it on the gateway channel
pub(crate) struct GatewayNanny {
    graph: GraphWatcher,
    config: ConfigWatcher,
    sender: GatewaySender,
}

impl GatewayNanny {
    pub fn new(graph: GraphWatcher, config: ConfigWatcher, sender: GatewaySender) -> Self {
        Self { graph, config, sender }
    }

    pub async fn handler(self) {
        log::trace!("starting the gateway nanny");

        let streams: [BoxStream<'static, NannyMessage>; 2] = [
            Box::pin(WatchStream::new(self.graph.clone()).map(|_| NannyMessage::GraphUpdated)),
            Box::pin(WatchStream::new(self.config.clone()).map(|_| NannyMessage::ConfigUpdated)),
        ];

        let mut stream = streams.merge();

        while let Some(message) = stream.next().await {
            log::trace!("nanny received a {message:?}");
            let config = self.config.borrow();
            let gateway = new_gateway(self.graph.borrow().clone(), &config);
            if let Err(error) = self.sender.send(gateway) {
                log::error!("Couldn't publish new gateway: {error:?}");
            }
        }
    }
}

pub(super) fn new_gateway(graph: Option<FederatedGraph>, config: &FederatedGraphConfig) -> Option<Arc<Engine>> {
    let config = engine_config_builder::build_config(config, graph?);
    let async_runtime = runtime_local::TokioCurrentRuntime::runtime();
    let cache = runtime_local::InMemoryCache::runtime(async_runtime.clone());
    Some(Arc::new(Engine::new(
        config.into_latest().into(),
        ulid::Ulid::new().to_string().into(),
        EngineEnv {
            async_runtime,
            cache,
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
            trusted_documents: runtime_noop::trusted_documents::NoopTrustedDocuments.into(),
            cache_opeartion_cache_control: false,
            kv: runtime_local::InMemoryKvStore::runtime(),
        },
    )))
}

#[derive(Debug)]
enum NannyMessage {
    GraphUpdated,
    ConfigUpdated,
}
