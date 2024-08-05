use std::path::PathBuf;
use std::sync::Arc;

use runtime::entity_cache::EntityCache;
use runtime_local::rate_limiting::in_memory::key_based::InMemoryRateLimiter;
use runtime_local::rate_limiting::redis::RedisRateLimiter;
use runtime_local::redis::{RedisPoolFactory, RedisTlsConfig};
use tokio::sync::watch;

use engine_v2::Engine;
use graphql_composition::FederatedGraph;
use runtime_local::{ComponentLoader, HooksWasi, InMemoryEntityCache, InMemoryKvStore, RedisEntityCache};
use runtime_noop::trusted_documents::NoopTrustedDocuments;

use gateway_config::{Config, EntityCachingRedisConfig};

use crate::hot_reload::ConfigWatcher;

/// Send half of the gateway watch channel
#[cfg(not(feature = "lambda"))]
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Option<Arc<Engine<GatewayRuntime>>>>;

/// Creates a new gateway from federated schema.
pub(super) async fn generate(
    federated_schema: &str,
    branch_id: Option<ulid::Ulid>,
    gateway_config: &Config,
    hot_reload_config_path: Option<PathBuf>,
) -> crate::Result<Engine<GatewayRuntime>> {
    let schema_version = blake3::hash(federated_schema.as_bytes());
    let graph =
        FederatedGraph::from_sdl(federated_schema).map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;
    let config = engine_config_builder::build_with_toml_config(gateway_config, graph).into_latest();

    // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
    let trusted_documents = if gateway_config.trusted_documents.enabled {
        let Some(branch_id) = branch_id else {
            return Err(crate::Error::InternalError(
                "Trusted documents are not implemented yet in airgapped mode".into(),
            ));
        };

        runtime::trusted_documents_client::Client::new(super::trusted_documents_client::TrustedDocumentsClient {
            http_client: Default::default(),
            bypass_header: gateway_config
                .trusted_documents
                .bypass_header
                .bypass_header_name
                .as_ref()
                .zip(
                    gateway_config
                        .trusted_documents
                        .bypass_header
                        .bypass_header_value
                        .as_ref(),
                )
                .map(|(name, value)| (name.clone().into(), String::from(value.as_ref()))),
            branch_id,
        })
    } else {
        runtime::trusted_documents_client::Client::new(NoopTrustedDocuments)
    };

    let mut redis_factory = RedisPoolFactory::default();

    let watcher = ConfigWatcher::init(gateway_config.clone(), hot_reload_config_path)?;

    let rate_limiter = match config.rate_limit_config() {
        Some(config) if config.storage.is_redis() => {
            let tls = config.redis.tls.map(|tls| RedisTlsConfig {
                cert: tls.cert,
                key: tls.key,
                ca: tls.ca,
            });

            let pool = redis_factory
                .pool(config.redis.url, tls)
                .map_err(|e| crate::Error::InternalError(e.to_string()))?;

            let global_config = runtime_local::rate_limiting::redis::RateLimitRedisConfig {
                key_prefix: config.redis.key_prefix,
            };

            RedisRateLimiter::runtime(global_config, pool, watcher)
                .await
                .map_err(|e| crate::Error::InternalError(e.to_string()))?
        }
        _ => InMemoryRateLimiter::runtime_with_watcher(watcher),
    };

    let entity_cache: Box<dyn EntityCache> = match gateway_config.entity_caching.storage {
        gateway_config::EntityCachingStorage::Memory => Box::new(InMemoryEntityCache::default()),
        gateway_config::EntityCachingStorage::Redis => {
            let EntityCachingRedisConfig { url, key_prefix, tls } = &gateway_config.entity_caching.redis;
            let tls = tls.as_ref().map(|tls| RedisTlsConfig {
                cert: tls.cert.as_deref(),
                key: tls.key.as_deref(),
                ca: tls.ca.as_deref(),
            });

            let pool = redis_factory
                .pool(url.as_str(), tls)
                .map_err(|e| crate::Error::InternalError(e.to_string()))?;

            Box::new(RedisEntityCache::new(pool, key_prefix))
        }
    };

    let runtime = GatewayRuntime {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        kv: InMemoryKvStore::runtime(),
        trusted_documents,
        meter: grafbase_telemetry::metrics::meter_from_global_provider(),
        hooks: HooksWasi::new(
            gateway_config
                .hooks
                .clone()
                .map(ComponentLoader::new)
                .transpose()
                .map_err(|e| crate::Error::InternalError(e.to_string()))?
                .flatten(),
        ),
        rate_limiter,
        entity_cache,
    };

    let config = config
        .try_into()
        .map_err(|err| crate::Error::InternalError(format!("Failed to generate engine Schema: {err}")))?;

    Ok(Engine::new(Arc::new(config), Some(schema_version.as_bytes()), runtime).await)
}

pub struct GatewayRuntime {
    fetcher: runtime::fetch::Fetcher,
    trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    meter: grafbase_telemetry::otel::opentelemetry::metrics::Meter,
    hooks: HooksWasi,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
}

impl engine_v2::Runtime for GatewayRuntime {
    type Hooks = HooksWasi;
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
    fn hooks(&self) -> &HooksWasi {
        &self.hooks
    }
    fn cache_factory(&self) -> &Self::CacheFactory {
        &()
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    fn sleep(&self, duration: std::time::Duration) -> futures_util::future::BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(duration))
    }

    fn entity_cache(&self) -> &dyn EntityCache {
        self.entity_cache.as_ref()
    }
}
