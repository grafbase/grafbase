use std::sync::Arc;

use ::engine::{CachedOperation, Schema};
use extension_catalog::ExtensionCatalog;
use gateway_config::{Config, EntityCachingRedisConfig};
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{entity_cache::EntityCache, trusted_documents_client::TrustedDocumentsEnforcementMode};
use runtime_local::{
    InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCache, NativeFetcher, RedisEntityCache,
    operation_cache::{RedisOperationCache, TieredOperationCache},
    rate_limiting::{in_memory::key_based::InMemoryRateLimiter, redis::RedisRateLimiter},
    redis::{RedisPoolFactory, RedisTlsConfig},
};
use url::Url;
use wasi_component_loader::extension::EngineWasmExtensions;

use crate::{
    engine::{
        EngineBuildContext,
        trusted_documents_client::{TrustedDocumentsClient, TrustedDocumentsClientConfig},
    },
    graph::{Graph, object_storage_host},
    hot_reload::ConfigWatcher,
};

/// Represents the runtime environment for the gateway, managing various components
/// such as fetching, rate limiting, entity caching, and metrics collection.
pub struct EngineRuntime {
    fetcher: NativeFetcher,
    pub(super) trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    metrics: EngineMetrics,
    pub(crate) extensions: EngineWasmExtensions,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
    pub(crate) operation_cache: TieredOperationCache<Arc<CachedOperation>>,
    authentication: engine_auth::AuthenticationService<EngineWasmExtensions>,
}

impl EngineRuntime {
    pub(super) async fn build(
        ctx: EngineBuildContext<'_>,
        graph: &Graph,
        schema: &Arc<Schema>,
        extension_catalog: &ExtensionCatalog,
    ) -> Result<EngineRuntime, crate::Error> {
        tracing::debug!("Build engine runtime.");

        let mut redis_factory = RedisPoolFactory::default();
        let config_watcher = ConfigWatcher::init(ctx.gateway_config.clone(), ctx.hot_reload_config_path.cloned())?;
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();

        tracing::debug!("Building rate limiter");

        let rate_limiter = match &ctx.gateway_config.gateway.rate_limit {
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

                RedisRateLimiter::runtime(global_config, pool, config_watcher, &meter)
                    .await
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?
            }
            _ => InMemoryRateLimiter::runtime_with_watcher(config_watcher),
        };

        tracing::debug!("Building cache");

        let entity_cache: Box<dyn EntityCache> = match ctx.gateway_config.entity_caching.storage {
            gateway_config::EntityCachingStorage::Memory => Box::new(InMemoryEntityCache::default()),
            gateway_config::EntityCachingStorage::Redis => {
                let EntityCachingRedisConfig { url, key_prefix, tls } = &ctx.gateway_config.entity_caching.redis;

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

        let operation_cache = operation_cache(ctx.gateway_config, &mut redis_factory)?;

        tracing::debug!("Building extensions");

        let extensions = EngineWasmExtensions::new(
            extension_catalog,
            ctx.gateway_config,
            schema,
            ctx.logging_filter.to_string(),
        )
        .await
        .map_err(|e| crate::Error::InternalError(format!("Error building an extension: {e}")))?;

        let kv = InMemoryKvStore::runtime();

        tracing::debug!("Setting up authentication");

        let authentication =
            engine_auth::AuthenticationService::new(ctx.gateway_config, extension_catalog, extensions.clone(), &kv);

        let trusted_documents = if let Some((access_token, branch_id)) = ctx.access_token.zip(graph.branch_id()) {
            let cfg = &ctx.gateway_config.trusted_documents;
            let enforcement_mode = if cfg.enforced {
                TrustedDocumentsEnforcementMode::Enforce
            } else {
                TrustedDocumentsEnforcementMode::Allow
            };

            let bypass_header = cfg
                .bypass_header
                .bypass_header_name
                .as_ref()
                .zip(cfg.bypass_header.bypass_header_value.as_ref())
                .map(|(name, value)| (name.clone().into(), String::from(value.as_str())));

            runtime::trusted_documents_client::Client::new(TrustedDocumentsClient::new(TrustedDocumentsClientConfig {
                branch_id,
                bypass_header,
                enforcement_mode,
                object_storage_url: object_storage_host()
                    .parse::<Url>()
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?,
                access_token,
            }))
        } else {
            runtime::trusted_documents_client::Client::new(())
        };

        let runtime = EngineRuntime {
            fetcher: NativeFetcher::new(ctx.gateway_config)
                .map_err(|e| crate::Error::FetcherConfigError(e.to_string()))?,
            kv,
            trusted_documents,
            extensions,
            metrics: EngineMetrics::build(&meter, graph.version_id().map(|id| id.to_string())),
            rate_limiter,
            entity_cache,
            operation_cache,
            authentication,
        };

        Ok(runtime)
    }
}

impl engine::Runtime for EngineRuntime {
    type Fetcher = NativeFetcher;
    type OperationCache = TieredOperationCache<Arc<CachedOperation>>;
    type Extensions = EngineWasmExtensions;
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
