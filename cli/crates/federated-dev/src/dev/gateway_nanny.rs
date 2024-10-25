use std::{collections::HashMap, sync::Arc};

use crate::ConfigWatcher;

use super::bus::{EngineSender, GraphWatcher};
use engine_v2::Engine;
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use gateway_config::GraphRateLimit;
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::rate_limiting::RateLimitKey;
use runtime_local::{rate_limiting::in_memory::key_based::InMemoryRateLimiter, InMemoryEntityCache, NativeFetcher};
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
            let config = self
                .graph
                .borrow()
                .clone()
                .map(|graph| engine_config_builder::build_with_sdl_config(&self.config.borrow(), graph));
            let gateway = new_gateway(config).await;
            if let Err(error) = self.sender.send(gateway) {
                log::error!("Couldn't publish new gateway: {error:?}");
            }
        }
    }
}

pub(super) async fn new_gateway(config: Option<engine_v2::VersionedConfig>) -> Option<Arc<Engine<CliRuntime>>> {
    let config = config?.into_latest();
    let graph = &config.graph;

    let runtime = CliRuntime {
        fetcher: NativeFetcher::default(),
        trusted_documents: runtime::trusted_documents_client::Client::new(
            runtime_noop::trusted_documents::NoopTrustedDocuments,
        ),
        kv: runtime_local::InMemoryKvStore::runtime(),
        metrics: EngineMetrics::build(&grafbase_telemetry::metrics::meter_from_global_provider(), None),
        // FIXME: God is this ugly
        rate_limiter: InMemoryRateLimiter::runtime({
            let mut key_based_config = HashMap::new();

            if let Some(global_config) = config.rate_limit.as_ref().and_then(|c| c.global) {
                key_based_config.insert(
                    RateLimitKey::Global,
                    GraphRateLimit {
                        limit: global_config.limit,
                        duration: global_config.duration,
                    },
                );
            }

            for (subgraph_name, subgraph) in config.subgraph_configs.iter() {
                if let Some(limit) = subgraph.rate_limit {
                    let name = &graph[graph[*subgraph_name].name];
                    key_based_config.insert(
                        RateLimitKey::Subgraph(name.clone().into()),
                        GraphRateLimit {
                            limit: limit.limit,
                            duration: limit.duration,
                        },
                    );
                }
            }

            key_based_config
        }),
        entity_cache: InMemoryEntityCache::default(),
    };

    let schema = engine_v2::Schema::build(config, engine_v2::SchemaVersion::from(ulid::Ulid::new().to_bytes())).ok()?;
    let engine = Engine::new(Arc::new(schema), runtime).await;

    Some(Arc::new(engine))
}

pub struct CliRuntime {
    fetcher: NativeFetcher,
    trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    metrics: EngineMetrics,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: InMemoryEntityCache,
}

impl engine_v2::Runtime for CliRuntime {
    type Hooks = ();
    type Fetcher = NativeFetcher;
    type OperationCacheFactory = ();

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn metrics(&self) -> &EngineMetrics {
        &self.metrics
    }

    fn hooks(&self) -> &() {
        &()
    }

    fn operation_cache_factory(&self) -> &() {
        &()
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await
    }

    fn entity_cache(&self) -> &dyn runtime::entity_cache::EntityCache {
        &self.entity_cache
    }
}

#[derive(Debug)]
enum NannyMessage {
    GraphUpdated,
    ConfigUpdated,
}
