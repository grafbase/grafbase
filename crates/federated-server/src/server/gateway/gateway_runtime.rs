use std::path::PathBuf;

use gateway_config::{Config, EntityCachingRedisConfig};
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::entity_cache::EntityCache;
use runtime_local::{
    rate_limiting::{in_memory::key_based::InMemoryRateLimiter, redis::RedisRateLimiter},
    redis::{RedisPoolFactory, RedisTlsConfig},
    HooksWasi, InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCacheFactory, NativeFetcher, RedisEntityCache,
};
use runtime_noop::trusted_documents::NoopTrustedDocuments;

use crate::hot_reload::ConfigWatcher;

/// Represents the runtime environment for the gateway, managing various components
/// such as fetching, rate limiting, entity caching, and metrics collection.
pub struct GatewayRuntime {
    fetcher: NativeFetcher,
    pub(super) trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    metrics: EngineMetrics,
    hooks: HooksWasi,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
    operation_cache_factory: InMemoryOperationCacheFactory,
}

impl GatewayRuntime {
    pub(super) async fn build(
        gateway_config: &Config,
        hot_reload_config_path: Option<PathBuf>,
        config: &engine_v2::config::Config,
        version_id: Option<ulid::Ulid>,
        hooks: HooksWasi,
    ) -> Result<GatewayRuntime, crate::Error> {
        let mut redis_factory = RedisPoolFactory::default();
        let watcher = ConfigWatcher::init(gateway_config.clone(), hot_reload_config_path)?;
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
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

                RedisRateLimiter::runtime(global_config, pool, watcher, &meter)
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
            fetcher: NativeFetcher::new(gateway_config).map_err(|e| crate::Error::FetcherConfigError(e.to_string()))?,
            kv: InMemoryKvStore::runtime(),
            trusted_documents: runtime::trusted_documents_client::Client::new(NoopTrustedDocuments),
            hooks,
            metrics: EngineMetrics::build(&meter, version_id.map(|id| id.to_string())),
            rate_limiter,
            entity_cache,
            operation_cache_factory: InMemoryOperationCacheFactory::default(),
        };

        Ok(runtime)
    }
}

impl engine_v2::Runtime for GatewayRuntime {
    type Hooks = HooksWasi;
    type Fetcher = NativeFetcher;
    type OperationCacheFactory = InMemoryOperationCacheFactory;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn hooks(&self) -> &HooksWasi {
        &self.hooks
    }

    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory {
        &self.operation_cache_factory
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await
    }

    fn entity_cache(&self) -> &dyn EntityCache {
        self.entity_cache.as_ref()
    }

    fn metrics(&self) -> &grafbase_telemetry::metrics::EngineMetrics {
        &self.metrics
    }
}
