use std::collections::HashMap;
use std::sync::Arc;

use crate::ConfigWatcher;

use super::bus::{EngineSender, GraphWatcher};
use engine_v2::Engine;
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use runtime::rate_limiting::KeyedRateLimitConfig;
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
                .map(|graph| engine_config_builder::build_config(&self.config.borrow(), graph));
            let gateway = new_gateway(config).await;
            if let Err(error) = self.sender.send(gateway) {
                log::error!("Couldn't publish new gateway: {error:?}");
            }
        }
    }
}

pub(super) async fn new_gateway(config: Option<engine_v2::VersionedConfig>) -> Option<Arc<Engine<CliRuntime>>> {
    let config = config?.into_latest();
    let rate_limiting_configs = config
        .as_keyed_rate_limit_config()
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                runtime::rate_limiting::RateLimitConfig {
                    limit: v.limit,
                    duration: v.duration,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let runtime = CliRuntime {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        trusted_documents: runtime::trusted_documents_client::Client::new(
            runtime_noop::trusted_documents::NoopTrustedDocuments,
        ),
        kv: runtime_local::InMemoryKvStore::runtime(),
        meter: grafbase_telemetry::metrics::meter_from_global_provider(),
        rate_limiter: runtime_local::rate_limiting::key_based::InMemoryRateLimiter::runtime(KeyedRateLimitConfig {
            rate_limiting_configs,
        }),
    };

    let schema = config.try_into().ok()?;
    let engine = Engine::new(Arc::new(schema), None, runtime).await;

    Some(Arc::new(engine))
}

pub struct CliRuntime {
    fetcher: runtime::fetch::Fetcher,
    trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    meter: grafbase_telemetry::otel::opentelemetry::metrics::Meter,
    rate_limiter: runtime::rate_limiting::RateLimiter,
}

impl engine_v2::Runtime for CliRuntime {
    type Hooks = ();
    type CacheFactory = ();

    fn fetcher(&self) -> &runtime::fetch::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn meter(&self) -> &grafbase_telemetry::otel::opentelemetry::metrics::Meter {
        &self.meter
    }

    fn hooks(&self) -> &() {
        &()
    }

    fn cache_factory(&self) -> &() {
        &()
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }
}

#[derive(Debug)]
enum NannyMessage {
    GraphUpdated,
    ConfigUpdated,
}
