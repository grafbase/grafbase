use std::sync::Arc;

use ::engine::{CachedOperation, Schema};
use extension_catalog::ExtensionCatalog;
use gateway_config::{EntityCachingRedisConfig, operation_caching::OperationCacheConfig};
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{entity_cache::EntityCache, trusted_documents_client::TrustedDocumentsEnforcementMode};
use runtime_local::{
    InMemoryEntityCache, InMemoryOperationCache, NativeFetcher, RedisEntityCache,
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
    metrics: EngineMetrics,
    pub(crate) extensions: EngineWasmExtensions,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
    entity_cache_config: gateway_config::EntityCachingConfig,
    pub(crate) operation_cache: TieredOperationCache<Arc<CachedOperation>>,
    operation_cache_config: OperationCacheConfig,
    redis_factory: Arc<tokio::sync::Mutex<RedisPoolFactory>>,
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
        let entity_cache = build_entity_cache(&ctx.gateway_config.entity_caching, &mut redis_factory)?;
        let operation_cache = build_operation_cache(&ctx.gateway_config.operation_caching, &mut redis_factory)?;

        tracing::debug!("Building extensions");

        let extensions = EngineWasmExtensions::new(
            ctx.gateway_extensions.clone(),
            extension_catalog,
            ctx.gateway_config,
            schema,
            ctx.logging_filter.to_string(),
        )
        .await
        .map_err(|e| crate::Error::InternalError(format!("Error building an extension: {e}")))?;

        tracing::debug!("Setting up authentication");

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
            trusted_documents,
            extensions,
            metrics: EngineMetrics::build(&meter, graph.version_id().map(|id| id.to_string())),
            rate_limiter,
            entity_cache,
            entity_cache_config: ctx.gateway_config.entity_caching.clone(),
            operation_cache,
            operation_cache_config: ctx.gateway_config.operation_caching.clone(),
            redis_factory: Arc::new(tokio::sync::Mutex::new(redis_factory)),
        };

        Ok(runtime)
    }
}

impl engine::Runtime for EngineRuntime {
    type Fetcher = NativeFetcher;
    type OperationCache = TieredOperationCache<Arc<CachedOperation>>;
    type Extensions = EngineWasmExtensions;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
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

    async fn clone_and_adjust_for_contract(&self, schema: &Arc<Schema>) -> Result<Self, String> {
        let mut redis_facttory = self.redis_factory.lock().await;
        let entity_cache = build_entity_cache(&self.entity_cache_config, &mut redis_facttory)
            .map_err(|err| format!("Failed to build entity cache: {err}"))?;
        let operation_cache = build_operation_cache(&self.operation_cache_config, &mut redis_facttory)
            .map_err(|err| format!("Failed to build operation cache: {err}"))?;
        Ok(EngineRuntime {
            fetcher: self.fetcher.clone(),
            trusted_documents: self.trusted_documents.clone(),
            metrics: self.metrics.clone(),
            extensions: self
                .extensions
                .clone_and_adjust_for_contract(schema)
                .await
                .map_err(|err| format!("Failed to adjust extensions for contract: {err}"))?,
            rate_limiter: self.rate_limiter.clone(),
            entity_cache,
            entity_cache_config: self.entity_cache_config.clone(),
            operation_cache,
            operation_cache_config: self.operation_cache_config.clone(),
            redis_factory: self.redis_factory.clone(),
        })
    }
}

fn build_entity_cache(
    config: &gateway_config::EntityCachingConfig,
    redis_factory: &mut RedisPoolFactory,
) -> Result<Box<dyn EntityCache>, crate::Error> {
    Ok(match config.storage {
        gateway_config::EntityCachingStorage::Memory => Box::new(InMemoryEntityCache::default()),
        gateway_config::EntityCachingStorage::Redis => {
            let EntityCachingRedisConfig { url, key_prefix, tls } = &config.redis;
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
    })
}

fn build_operation_cache(
    config: &OperationCacheConfig,
    redis_factory: &mut RedisPoolFactory,
) -> Result<TieredOperationCache<Arc<CachedOperation>>, crate::Error> {
    Ok(match (config.enabled, config.redis.as_ref()) {
        (false, _) => TieredOperationCache::new(InMemoryOperationCache::inactive(), None),
        (true, None) => TieredOperationCache::new(InMemoryOperationCache::new(config.limit), None),
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
                InMemoryOperationCache::new(config.limit),
                Some(RedisOperationCache::new(pool, &redis_config.key_prefix)),
            )
        }
    })
}
