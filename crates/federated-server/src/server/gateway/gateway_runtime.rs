use std::{path::PathBuf, sync::Arc};

use engine::{CachedOperation, Schema};
use extension_catalog::ExtensionCatalog;
use gateway_config::{Config, EntityCachingRedisConfig};
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::entity_cache::EntityCache;
use runtime_local::{
    InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCache, NativeFetcher, RedisEntityCache,
    operation_cache::{RedisOperationCache, TieredOperationCache},
    rate_limiting::{in_memory::key_based::InMemoryRateLimiter, redis::RedisRateLimiter},
    redis::{RedisPoolFactory, RedisTlsConfig},
};
use wasi_component_loader::extension::WasmExtensions;

use crate::hot_reload::ConfigWatcher;

pub(super) struct GatewayRuntimeConfig<'a> {
    pub(super) gateway_config: &'a Config,
    pub(super) extension_catalog: &'a ExtensionCatalog,
    pub(super) schema: &'a Arc<Schema>,
    pub(super) hot_reload_config_path: Option<PathBuf>,
    pub(super) version_id: Option<ulid::Ulid>,
    pub(super) logging_filter: String,
}

/// Represents the runtime environment for the gateway, managing various components
/// such as fetching, rate limiting, entity caching, and metrics collection.
pub struct GatewayRuntime {
    fetcher: NativeFetcher,
    pub(super) trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    metrics: EngineMetrics,
    pub(crate) extensions: WasmExtensions,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
    pub(crate) operation_cache: TieredOperationCache<Arc<CachedOperation>>,
    authentication: engine_auth::AuthenticationService<WasmExtensions>,
}

impl GatewayRuntime {
    pub(super) async fn build(config: GatewayRuntimeConfig<'_>) -> Result<GatewayRuntime, crate::Error> {
        let GatewayRuntimeConfig {
            gateway_config,
            extension_catalog,
            schema,
            hot_reload_config_path,
            version_id,
            logging_filter,
        } = config;

        tracing::debug!("Build engine runtime.");

        let mut redis_factory = RedisPoolFactory::default();
        let watcher = ConfigWatcher::init(gateway_config.clone(), hot_reload_config_path)?;
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();

        tracing::debug!("Building rate limiter");

        let rate_limiter = match &gateway_config.gateway.rate_limit {
            Some(config) if config.storage.is_redis() => {
                let tls = config.redis.tls.as_ref().map(|tls| RedisTlsConfig {
                    cert: tls.cert.as_deref(),
                    key: tls.key.as_deref(),
                    ca: tls.ca.as_deref(),
                });

                let pool = redis_factory
                    .pool(config.redis.url.as_str(), tls)
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                let global_config = runtime_local::rate_limiting::redis::RateLimitRedisConfig {
                    key_prefix: &config.redis.key_prefix,
                };

                RedisRateLimiter::runtime(global_config, pool, watcher, &meter)
                    .await
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?
            }
            _ => InMemoryRateLimiter::runtime_with_watcher(watcher),
        };

        tracing::debug!("Building cache");

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

        let operation_cache = operation_cache(gateway_config, &mut redis_factory)?;

        tracing::debug!("Building extensions");

        let extensions = WasmExtensions::new(extension_catalog, gateway_config, schema, logging_filter)
            .await
            .map_err(|e| crate::Error::InternalError(format!("Error building an extension: {e}")))?;

        let kv = InMemoryKvStore::runtime();

        tracing::debug!("Setting up authentication");

        let authentication =
            engine_auth::AuthenticationService::new(gateway_config, extension_catalog, extensions.clone(), &kv);

        let runtime = GatewayRuntime {
            fetcher: NativeFetcher::new(gateway_config).map_err(|e| crate::Error::FetcherConfigError(e.to_string()))?,
            kv,
            trusted_documents: runtime::trusted_documents_client::Client::new(()),
            extensions,
            metrics: EngineMetrics::build(&meter, version_id.map(|id| id.to_string())),
            rate_limiter,
            entity_cache,
            operation_cache,
            authentication,
        };

        Ok(runtime)
    }
}

impl engine::Runtime for GatewayRuntime {
    type Fetcher = NativeFetcher;
    type OperationCache = TieredOperationCache<Arc<CachedOperation>>;
    type Extensions = WasmExtensions;
    type Authenticate = engine_auth::AuthenticationService<Self::Extensions>;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn operation_cache(&self) -> &Self::OperationCache {
        &self.operation_cache
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

    fn extensions(&self) -> &Self::Extensions {
        &self.extensions
    }

    fn authentication(&self) -> &Self::Authenticate {
        &self.authentication
    }
}

fn operation_cache(
    gateway_config: &Config,
    redis_factory: &mut RedisPoolFactory,
) -> Result<TieredOperationCache<Arc<CachedOperation>>, crate::Error> {
    Ok(
        match (
            gateway_config.operation_caching.enabled,
            gateway_config.operation_caching.redis.as_ref(),
        ) {
            (false, _) => TieredOperationCache::new(InMemoryOperationCache::inactive(), None),
            (true, None) => TieredOperationCache::new(
                InMemoryOperationCache::new(gateway_config.operation_caching.limit),
                None,
            ),
            (true, Some(redis_config)) => {
                let tls = redis_config.tls.as_ref().map(|tls| RedisTlsConfig {
                    cert: tls.cert.as_deref(),
                    key: tls.key.as_deref(),
                    ca: tls.ca.as_deref(),
                });

                let pool = redis_factory
                    .pool(redis_config.url.as_ref(), tls)
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                TieredOperationCache::new(
                    InMemoryOperationCache::new(gateway_config.operation_caching.limit),
                    Some(RedisOperationCache::new(pool, &redis_config.key_prefix)),
                )
            }
        },
    )
}
