use std::sync::Arc;

use crate::ConfigWatcher;

use super::bus::{EngineSender, GraphWatcher};
use engine_v2::{Engine, EngineEnv};
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use runtime::hooks::Hooks;
use runtime_noop::hooks::HooksNoop;
use tokio_stream::wrappers::WatchStream;

/// The GatewayNanny looks after the `Gateway` - on updates to the graph or config it'll
/// create a new `Gateway` and publish it on the gateway channel
pub(crate) struct EngineNanny {
    graph: GraphWatcher,
    config: ConfigWatcher,
    sender: EngineSender,
}

impl EngineNanny {
    pub fn new(graph: GraphWatcher, config: ConfigWatcher, sender: EngineSender) -> Self {
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

    let cache = runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
        common_cache_tags: vec![],
        enabled: true,
        subdomain: "localhost".to_string(),
    });

    let engine_env = EngineEnv {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        cache: cache.clone(),
        trusted_documents: runtime::trusted_documents_client::Client::new(
            runtime_noop::trusted_documents::NoopTrustedDocuments,
        ),
        kv: runtime_local::InMemoryKvStore::runtime(),
        meter: grafbase_tracing::metrics::meter_from_global_provider(),
        user_hooks: Hooks::new(HooksNoop),
    };

    let config = config.into_latest().try_into().ok()?;
    let engine = Engine::new(Arc::new(config), engine_env);

    Some(Arc::new(engine))
}

#[derive(Debug)]
enum NannyMessage {
    GraphUpdated,
    ConfigUpdated,
}
